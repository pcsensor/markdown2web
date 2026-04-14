use std::fs;
use std::sync::Arc;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
    response::Response,
};
use markdown2web::{
    app,
    config::AppConfig,
    content::markdown::render_markdown,
    store::{filesystem, sqlite::AppDatabase},
};
use regex::Regex;
use serde_json::{Value, json};
use tempfile::TempDir;
use tower::util::ServiceExt;

fn test_config(temp: &TempDir) -> AppConfig {
    let root = temp.path();
    let content_dir = root.join("content");
    let generated_dir = root.join("generated/site");
    let data_dir = root.join("data");
    AppConfig {
        host: "127.0.0.1".into(),
        port: 0,
        base_url: "http://127.0.0.1:0".into(),
        site_name: "Test markdown2web".into(),
        notes_dir: content_dir.join("notes"),
        assets_dir: content_dir.join("assets"),
        generated_assets_dir: generated_dir.join("assets"),
        content_dir,
        generated_dir,
        data_dir,
        admin_username: "admin".into(),
        admin_password: "admin123456".into(),
        watch_enabled: false,
        turnstile_enabled: false,
        secure_cookies: false,
        session_ttl_hours: 168,
        upload_limit_mb: 10,
        turnstile_site_key: String::new(),
        turnstile_secret_key: String::new(),
    }
}

async fn setup() -> (TempDir, app::AppState, axum::Router) {
    setup_with_upload_limit(10).await
}

async fn setup_with_upload_limit(upload_limit_mb: usize) -> (TempDir, app::AppState, axum::Router) {
    let temp = TempDir::new().unwrap();
    let mut config = test_config(&temp);
    config.upload_limit_mb = upload_limit_mb;
    setup_with_config(temp, config).await
}

async fn setup_with_config(
    temp: TempDir,
    config: AppConfig,
) -> (TempDir, app::AppState, axum::Router) {
    config.ensure_directories().unwrap();
    let db = Arc::new(AppDatabase::open(&config.database_path()).unwrap());
    db.initialize(&config.admin_username, &config.admin_password)
        .unwrap();
    let state = app::AppState::bootstrap(config, db).await.unwrap();
    let router = app::build_router(state.clone());
    (temp, state, router)
}

#[test]
fn initialize_does_not_reset_existing_admin_password_without_explicit_sync() {
    let temp = TempDir::new().unwrap();
    let db = AppDatabase::open(&temp.path().join("app.db")).unwrap();

    db.initialize("admin", "old-password").unwrap();
    assert!(db.verify_user("admin", "old-password").unwrap());

    db.initialize("admin", "new-password").unwrap();
    assert!(db.verify_user("admin", "old-password").unwrap());
    assert!(!db.verify_user("admin", "new-password").unwrap());
}

#[test]
fn explicit_admin_password_sync_updates_existing_admin_password() {
    let temp = TempDir::new().unwrap();
    let db = AppDatabase::open(&temp.path().join("app.db")).unwrap();

    db.initialize("admin", "old-password").unwrap();
    let session = db.create_session("admin", 168).unwrap();
    assert!(db.session_user(&session.token).unwrap().is_some());

    db.initialize_with_admin_password_sync("admin", "new-password", true)
        .unwrap();
    assert!(!db.verify_user("admin", "old-password").unwrap());
    assert!(db.verify_user("admin", "new-password").unwrap());
    assert!(db.session_user(&session.token).unwrap().is_none());
}

fn cookie_header(response: &Response) -> String {
    response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .filter_map(|value| value.split(';').next())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join("; ")
}

async fn response_html(response: Response) -> String {
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(body.to_vec()).unwrap()
}

fn csrf_from_html(html: &str) -> String {
    Regex::new(r#"name="_csrf" value="([^"]+)""#)
        .unwrap()
        .captures(html)
        .and_then(|caps| caps.get(1))
        .map(|capture| capture.as_str().to_string())
        .expect("csrf token should be rendered")
}

fn csrf_from_note_html(html: &str) -> String {
    Regex::new(r#"data-csrf-token="([^"]*)""#)
        .unwrap()
        .captures(html)
        .and_then(|caps| caps.get(1))
        .map(|capture| capture.as_str().to_string())
        .expect("note csrf token should be rendered")
}

async fn admin_login(router: &axum::Router) -> String {
    let response = admin_login_response(router, "admin123456").await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    cookie_header(&response)
}

async fn admin_login_response(router: &axum::Router, password: &str) -> Response {
    let login_page = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin/login")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let pre_auth_cookie = cookie_header(&login_page);
    let html = response_html(login_page).await;
    let csrf = csrf_from_html(&html);

    let login_request = Request::builder()
        .method("POST")
        .uri("/admin/login")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, pre_auth_cookie)
        .body(Body::from(format!(
            "username=admin&password={password}&_csrf={csrf}"
        )))
        .unwrap();
    router.clone().oneshot(login_request).await.unwrap()
}

async fn admin_page(router: &axum::Router, cookie: &str, uri: &str) -> (String, String) {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri(uri)
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let html = response_html(response).await;
    let csrf = csrf_from_html(&html);
    (html, csrf)
}

async fn public_register(
    router: &axum::Router,
    username: &str,
    password: &str,
    next: &str,
) -> String {
    let (pre_auth_cookie, csrf) = account_pre_auth(router, next).await;
    let request = Request::builder()
        .method("POST")
        .uri("/account/register")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, pre_auth_cookie)
        .body(Body::from(format!(
            "username={username}&password={password}&next={next}&_csrf={csrf}"
        )))
        .unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    cookie_header(&response)
}

async fn public_login(
    router: &axum::Router,
    username: &str,
    password: &str,
    next: &str,
) -> Response {
    let (pre_auth_cookie, csrf) = account_pre_auth(router, next).await;
    let request = Request::builder()
        .method("POST")
        .uri("/account/login")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, pre_auth_cookie)
        .body(Body::from(format!(
            "username={username}&password={password}&next={next}&_csrf={csrf}"
        )))
        .unwrap();
    router.clone().oneshot(request).await.unwrap()
}

async fn account_pre_auth(router: &axum::Router, next: &str) -> (String, String) {
    let account_page = router
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/account?next={next}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let pre_auth_cookie = cookie_header(&account_page);
    let html = response_html(account_page).await;
    let csrf = csrf_from_html(&html);
    (pre_auth_cookie, csrf)
}

async fn note_csrf(router: &axum::Router, cookie: &str, slug: &str) -> String {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/notes/{slug}"))
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    csrf_from_note_html(&response_html(response).await)
}

#[tokio::test]
async fn home_and_note_routes_render_content() {
    let (_temp, _state, router) = setup().await;

    let response = router
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("Welcome to markdown2web"));
    assert!(html.contains("reading-progress"));
    assert!(html.contains("/static/js/site.js"));
    assert!(html.contains("cursor-beacon"));
    assert!(html.contains("hero-panel panel interactive-card hero-mascot-panel"));
    assert!(html.contains("data-mascot"));
    assert!(html.contains("mascot-stage"));
    assert!(!html.contains("hero-particle-canvas"));
    assert!(html.contains("metric-card interactive-card interactive-card-subtle"));

    let response = router
        .oneshot(
            Request::builder()
                .uri("/notes/welcome")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("/notes/architecture"));
    assert!(html.contains("sidebar-card interactive-card sidebar-panel"));
    assert!(html.contains("data-note-article"));
    assert!(html.contains("data-annotation-toolbar"));
    assert!(html.contains("data-annotation-comment-lane"));
    assert!(html.contains("data-annotation-enabled=\"false\""));
    assert!(html.contains("/account?next=/notes/welcome"));
}

#[tokio::test]
async fn admin_auth_guard_and_save_note() {
    let (_temp, _state, router) = setup().await;

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(header::LOCATION).unwrap(),
        "/admin/login"
    );

    let login_page = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin/login")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(login_page.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("auth-panel"));

    let cookie = admin_login(&router).await;

    let (html, csrf) = admin_page(&router, &cookie, "/admin").await;
    assert!(html.contains("修改管理员密码"));
    assert!(html.contains("action=\"/admin/password\""));
    assert!(html.contains("href=\"/admin/users\""));
    assert!(html.contains("用户管理"));

    let save_request = Request::builder()
        .method("POST")
        .uri("/admin/notes/save")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, &cookie)
        .body(Body::from(format!("title=Integration%20Note&summary=Saved%20from%20test&tags=testing,axum&status=published&aliases=&body=%23%20Integration%20Note%0A%0AHello%20from%20test&_csrf={csrf}")))
        .unwrap();
    let response = router.clone().oneshot(save_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/notes/integration-note")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("Integration Note"));
}

#[tokio::test]
async fn admin_can_upload_mp4_asset() {
    let (_temp, state, router) = setup().await;

    let cookie = admin_login(&router).await;
    let (_html, csrf) = admin_page(&router, &cookie, "/admin").await;

    let boundary = "M2WBOUNDARY";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"_csrf\"\r\n\r\n{csrf}\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"demo-upload.mp4\"\r\nContent-Type: video/mp4\r\n\r\nfake-mp4\r\n--{boundary}--\r\n"
    );
    let upload_request = Request::builder()
        .method("POST")
        .uri("/admin/upload/asset")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .header(header::COOKIE, &cookie)
        .header("x-csrf-token", &csrf)
        .body(Body::from(body))
        .unwrap();
    let response = router.oneshot(upload_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get(header::LOCATION).unwrap(), "/admin");

    let uploaded = state.config.assets_dir.join("demo-upload.mp4");
    assert_eq!(fs::read(uploaded).unwrap(), b"fake-mp4");
}

#[tokio::test]
async fn admin_can_upload_mp3_asset() {
    let (_temp, state, router) = setup().await;

    let cookie = admin_login(&router).await;
    let (_html, csrf) = admin_page(&router, &cookie, "/admin").await;

    let boundary = "M2WBOUNDARY";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"_csrf\"\r\n\r\n{csrf}\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"demo-upload.mp3\"\r\nContent-Type: audio/mpeg\r\n\r\nfake-mp3\r\n--{boundary}--\r\n"
    );
    let upload_request = Request::builder()
        .method("POST")
        .uri("/admin/upload/asset")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .header(header::COOKIE, &cookie)
        .header("x-csrf-token", &csrf)
        .body(Body::from(body))
        .unwrap();
    let response = router.oneshot(upload_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get(header::LOCATION).unwrap(), "/admin");

    let uploaded = state.config.assets_dir.join("demo-upload.mp3");
    assert_eq!(fs::read(uploaded).unwrap(), b"fake-mp3");
}

#[tokio::test]
async fn admin_can_upload_mp3_larger_than_default_body_limit() {
    let (_temp, state, router) = setup().await;

    let cookie = admin_login(&router).await;
    let (_html, csrf) = admin_page(&router, &cookie, "/admin").await;

    let boundary = "M2WLARGEMP3";
    let mut body = Vec::new();
    body.extend_from_slice(
        format!("--{boundary}\r\nContent-Disposition: form-data; name=\"_csrf\"\r\n\r\n{csrf}\r\n")
            .as_bytes(),
    );
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"large-upload.mp3\"\r\nContent-Type: audio/mpeg\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend(std::iter::repeat_n(b'a', 3 * 1024 * 1024));
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    let upload_request = Request::builder()
        .method("POST")
        .uri("/admin/upload/asset")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .header(header::COOKIE, &cookie)
        .header("x-csrf-token", &csrf)
        .body(Body::from(body))
        .unwrap();
    let response = router.oneshot(upload_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let uploaded = state.config.assets_dir.join("large-upload.mp3");
    assert_eq!(fs::metadata(uploaded).unwrap().len(), 3 * 1024 * 1024);
}

#[tokio::test]
async fn admin_can_upload_32mb_mp4_when_limit_allows() {
    let (_temp, state, router) = setup_with_upload_limit(64).await;

    let cookie = admin_login(&router).await;
    let (_html, csrf) = admin_page(&router, &cookie, "/admin").await;

    let boundary = "M2W32MBMP4";
    let mut body = Vec::new();
    body.extend_from_slice(
        format!("--{boundary}\r\nContent-Disposition: form-data; name=\"_csrf\"\r\n\r\n{csrf}\r\n")
            .as_bytes(),
    );
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"large-video.mp4\"\r\nContent-Type: video/mp4\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend(std::iter::repeat_n(b'v', 32 * 1024 * 1024));
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    let upload_request = Request::builder()
        .method("POST")
        .uri("/admin/upload/asset")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .header(header::COOKIE, &cookie)
        .header("x-csrf-token", &csrf)
        .body(Body::from(body))
        .unwrap();
    let response = router.oneshot(upload_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let uploaded = state.config.assets_dir.join("large-video.mp4");
    assert_eq!(fs::metadata(uploaded).unwrap().len(), 32 * 1024 * 1024);
}

#[tokio::test]
async fn admin_upload_over_limit_does_not_return_internal_error() {
    let (_temp, _state, router) = setup_with_upload_limit(1).await;

    let cookie = admin_login(&router).await;
    let (_html, csrf) = admin_page(&router, &cookie, "/admin").await;

    let boundary = "M2WOVERLIMIT";
    let mut body = Vec::new();
    body.extend_from_slice(
        format!("--{boundary}\r\nContent-Disposition: form-data; name=\"_csrf\"\r\n\r\n{csrf}\r\n")
            .as_bytes(),
    );
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"too-large.mp3\"\r\nContent-Type: audio/mpeg\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend(std::iter::repeat_n(b'a', 3 * 1024 * 1024));
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    let upload_request = Request::builder()
        .method("POST")
        .uri("/admin/upload/asset")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .header(header::COOKIE, &cookie)
        .header("x-csrf-token", &csrf)
        .body(Body::from(body))
        .unwrap();
    let response = router.oneshot(upload_request).await.unwrap();
    assert_ne!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn csrf_is_required_for_state_changing_requests() {
    let (_temp, _state, router) = setup().await;
    let admin_cookie = admin_login(&router).await;

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/rebuild")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let user_cookie =
        public_register(&router, "csrf-user", "ReaderPass123", "/notes/welcome").await;
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/notes/welcome/annotations")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &user_cookie)
                .body(Body::from(
                    json!({
                        "start_offset": 0,
                        "end_offset": 7,
                        "quote": "Welcome",
                        "comment": "missing csrf"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn admin_rejects_unsafe_slugs_filenames_and_svg_uploads() {
    let (_temp, state, router) = setup().await;
    let admin_cookie = admin_login(&router).await;
    let (_html, csrf) = admin_page(&router, &admin_cookie, "/admin").await;

    let save_request = Request::builder()
        .method("POST")
        .uri("/admin/notes/save")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, &admin_cookie)
        .body(Body::from(format!(
            "title=Unsafe&slug=../outside&summary=x&tags=&status=published&aliases=&body=body&_csrf={csrf}"
        )))
        .unwrap();
    let response = router.clone().oneshot(save_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(!state.config.notes_dir.join("../outside.md").exists());

    let boundary = "M2WUNSAFE";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"_csrf\"\r\n\r\n{csrf}\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"../evil.mp3\"\r\nContent-Type: audio/mpeg\r\n\r\nfake\r\n--{boundary}--\r\n"
    );
    let upload_request = Request::builder()
        .method("POST")
        .uri("/admin/upload/asset")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .header(header::COOKIE, &admin_cookie)
        .header("x-csrf-token", &csrf)
        .body(Body::from(body))
        .unwrap();
    let response = router.clone().oneshot(upload_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let boundary = "M2WSVG";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"_csrf\"\r\n\r\n{csrf}\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"unsafe.svg\"\r\nContent-Type: image/svg+xml\r\n\r\n<svg></svg>\r\n--{boundary}--\r\n"
    );
    let upload_request = Request::builder()
        .method("POST")
        .uri("/admin/upload/asset")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .header(header::COOKIE, &admin_cookie)
        .header("x-csrf-token", &csrf)
        .body(Body::from(body))
        .unwrap();
    let response = router.oneshot(upload_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn media_embed_label_and_src_are_escaped() {
    let (html, _) = render_markdown(
        r#"#[<img src=x onerror=alert(1)>](/assets/sound"bad.mp3)

@[<script>alert(1)</script>](/assets/movie"bad.mp4)"#,
    )
    .unwrap();

    assert!(html.contains("&lt;img src=x onerror=alert(1)&gt;"));
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(html.contains("/assets/sound&quot;bad.mp3"));
    assert!(html.contains("/assets/movie&quot;bad.mp4"));
    assert!(!html.contains("<script>alert(1)</script>"));
}

#[tokio::test]
async fn expired_sessions_are_rejected_and_secure_cookie_is_configurable() {
    let temp = TempDir::new().unwrap();
    let mut config = test_config(&temp);
    config.secure_cookies = true;
    let (_temp, state, router) = setup_with_config(temp, config).await;

    let login_response = admin_login_response(&router, "admin123456").await;
    assert_eq!(login_response.status(), StatusCode::SEE_OTHER);
    let set_cookie = login_response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(set_cookie.contains("Secure"));

    state
        .db
        .create_public_user("expired-user", "ReaderPass123")
        .unwrap();
    let expired = state.db.create_public_session("expired-user", -1).unwrap();
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/notes/welcome/danmaku")
                .header(header::CONTENT_TYPE, "application/json")
                .header(
                    header::COOKIE,
                    format!("m2w_user_session={}", expired.token),
                )
                .header("x-csrf-token", expired.csrf_token)
                .body(Body::from(
                    json!({
                        "video_src": "/assets/demo.mp4#0",
                        "time_ms": 100,
                        "body": "expired"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn admin_login_is_rate_limited() {
    let (_temp, _state, router) = setup().await;

    for _ in 0..5 {
        let response = admin_login_response(&router, "wrong-password").await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    let response = admin_login_response(&router, "wrong-password").await;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn note_audio_embed_renders_and_asset_route_is_reachable() {
    let (_temp, state, router) = setup().await;
    fs::write(state.config.assets_dir.join("voice.mp3"), b"fake-mp3").unwrap();
    filesystem::write_note(
        &state.config,
        "audio-note",
        r#"---
title: Audio Note
slug: audio-note
summary: audio embed regression test
status: published
---
# Audio Note

#[试听](/assets/voice.mp3)
"#,
    )
    .unwrap();
    state
        .build_service
        .rebuild("test audio embed")
        .await
        .unwrap();

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/notes/audio-note")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("data-audio-player"));
    assert!(html.contains("audio-icon-play"));
    assert!(html.contains("audio-icon-pause"));
    assert!(html.contains("00:00/00:00"));

    let site = state.site.read().await.clone();
    let asset = site
        .assets
        .iter()
        .find(|asset| asset.public_url.ends_with("voice.mp3"))
        .expect("audio asset should be materialized");
    assert!(html.contains(&asset.public_url));

    let source_re = Regex::new(r#"source src="([^"]+voice\.mp3)""#).unwrap();
    let source = source_re
        .captures(&html)
        .and_then(|caps| caps.get(1))
        .map(|capture| capture.as_str().to_string())
        .expect("audio source should be rendered");
    assert_eq!(source, asset.public_url);

    let asset_response = router
        .oneshot(Request::builder().uri(&source).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(asset_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn note_video_embed_renders_and_asset_route_is_reachable() {
    let (_temp, state, router) = setup().await;
    fs::write(state.config.assets_dir.join("demo-video.mp4"), b"fake-mp4").unwrap();
    filesystem::write_note(
        &state.config,
        "video-note",
        r#"---
title: Video Note
slug: video-note
summary: video embed regression test
status: published
---
# Video Note

@[演示视频](/assets/demo-video.mp4)
"#,
    )
    .unwrap();
    let summary = state
        .build_service
        .rebuild("admin save video-note")
        .await
        .unwrap();
    assert!(
        summary.media_jobs.is_empty(),
        "admin note saves should not start media processing when media files did not change"
    );

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/notes/video-note")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("data-video-player"));
    assert!(html.contains("video-player-frame"));
    assert!(html.contains("video-player-media"));
    assert!(html.contains("class=\"video-player-media\" preload=\"none\" playsinline"));
    assert!(html.contains("data-video-load data-static-button"));
    assert!(html.contains("data-video-controls"));
    assert!(html.contains("data-video-toggle"));
    assert!(html.contains("data-video-progress"));
    assert!(html.contains("data-video-volume"));
    assert!(html.contains("data-video-volume-label"));
    assert!(html.contains("data-video-fullscreen"));
    assert!(html.contains("video-fullscreen-button"));
    assert!(html.contains("data-video-danmaku-color"));
    assert!(html.contains("data-video-danmaku-layer"));
    assert!(html.contains("data-video-danmaku-form"));
    assert!(html.contains("data-video-danmaku-input"));
    assert!(html.contains("data-video-danmaku-login"));
    assert!(html.contains("data-video-speed"));
    assert!(html.contains("data-video-danmaku-size"));
    assert!(html.contains("1.25rem\" selected"));
    assert!(html.contains("data-video-key"));
    assert!(!html.contains(" controls "));
    assert!(html.contains("无法播放视频：演示视频"));
    assert!(!html.contains("video-label"));

    let site = state.site.read().await.clone();
    let asset = site
        .assets
        .iter()
        .find(|asset| asset.public_url.ends_with("demo-video.mp4"))
        .expect("video asset should be materialized");
    assert!(html.contains(&asset.public_url));

    let source_re = Regex::new(r#"source data-src="([^"]+demo-video\.mp4)""#).unwrap();
    let source = source_re
        .captures(&html)
        .and_then(|caps| caps.get(1))
        .map(|capture| capture.as_str().to_string())
        .expect("video source should be rendered");
    assert_eq!(source, asset.public_url);

    let asset_response = router
        .oneshot(Request::builder().uri(&source).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(asset_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn admin_can_change_password_and_old_password_stops_working() {
    let (_temp, _state, router) = setup().await;

    let cookie = admin_login(&router).await;

    let (html, csrf) = admin_page(&router, &cookie, "/admin").await;
    assert!(html.contains("修改管理员密码"));
    assert!(html.contains("action=\"/admin/password\""));

    let invalid_change = Request::builder()
        .method("POST")
        .uri("/admin/password")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, &cookie)
        .body(Body::from(format!(
            "current_password=wrong-password&new_password=NewSecurePass123&confirm_password=NewSecurePass123&_csrf={csrf}"
        )))
        .unwrap();
    let response = router.clone().oneshot(invalid_change).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("当前密码不正确"));

    let too_short_change = Request::builder()
        .method("POST")
        .uri("/admin/password")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, &cookie)
        .body(Body::from(format!(
            "current_password=admin123456&new_password=short&confirm_password=short&_csrf={csrf}"
        )))
        .unwrap();
    let response = router.clone().oneshot(too_short_change).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("新密码至少需要 8 个字符"));

    let mismatch_change = Request::builder()
        .method("POST")
        .uri("/admin/password")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, &cookie)
        .body(Body::from(format!(
            "current_password=admin123456&new_password=NewSecurePass123&confirm_password=OtherSecurePass123&_csrf={csrf}"
        )))
        .unwrap();
    let response = router.clone().oneshot(mismatch_change).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("两次输入的新密码不一致"));

    let change_password = Request::builder()
        .method("POST")
        .uri("/admin/password")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, &cookie)
        .body(Body::from(format!(
            "current_password=admin123456&new_password=NewSecurePass123&confirm_password=NewSecurePass123&_csrf={csrf}"
        )))
        .unwrap();
    let response = router.clone().oneshot(change_password).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(header::LOCATION).unwrap(),
        "/admin/login?password=updated"
    );
    let cleared_cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(cleared_cookie.contains("m2w_session="));

    let after_change = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(after_change.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        after_change.headers().get(header::LOCATION).unwrap(),
        "/admin/login"
    );

    let login_page = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin/login?password=updated")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(login_page.status(), StatusCode::OK);
    let body = to_bytes(login_page.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("密码已更新，请使用新密码重新登录"));

    let response = admin_login_response(&router, "admin123456").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("Invalid username or password"));

    let response = admin_login_response(&router, "NewSecurePass123").await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get(header::LOCATION).unwrap(), "/admin");
}

#[tokio::test]
async fn admin_can_manage_public_users() {
    let (_temp, _state, router) = setup().await;

    let admin_cookie = admin_login(&router).await;

    let (html, users_csrf) = admin_page(&router, &admin_cookie, "/admin/users").await;
    assert!(html.contains("用户管理"));
    assert!(html.contains("action=\"/admin/users\""));

    let create_user = Request::builder()
        .method("POST")
        .uri("/admin/users")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, &admin_cookie)
        .body(Body::from(format!(
            "username=reader-one&password=ReaderPass123&confirm_password=ReaderPass123&_csrf={users_csrf}"
        )))
        .unwrap();
    let response = router.clone().oneshot(create_user).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(header::LOCATION).unwrap(),
        "/admin/users/reader-one?status=created"
    );

    let create_second_user = Request::builder()
        .method("POST")
        .uri("/admin/users")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, &admin_cookie)
        .body(Body::from(format!(
            "username=reader-two&password=ReaderPass123&confirm_password=ReaderPass123&_csrf={users_csrf}"
        )))
        .unwrap();
    let response = router.clone().oneshot(create_second_user).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(header::LOCATION).unwrap(),
        "/admin/users/reader-two?status=created"
    );

    let (html, user_csrf) = admin_page(
        &router,
        &admin_cookie,
        "/admin/users/reader-one?status=created",
    )
    .await;
    assert!(html.contains("管理用户：reader-one"));
    assert!(html.contains("action=\"/admin/users/reader-one/update\""));
    assert!(html.contains("action=\"/admin/users/reader-one/delete\""));

    let duplicate_update = Request::builder()
        .method("POST")
        .uri("/admin/users/reader-one/update")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, &admin_cookie)
        .body(Body::from(format!(
            "username=reader-two&new_password=&confirm_password=&_csrf={user_csrf}"
        )))
        .unwrap();
    let response = router.clone().oneshot(duplicate_update).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("该用户名已被注册"));
    assert!(html.contains("action=\"/admin/users/reader-one/update\""));

    let response = public_login(&router, "reader-one", "ReaderPass123", "/notes/welcome").await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let reader_cookie = cookie_header(&response);
    let reader_csrf = note_csrf(&router, &reader_cookie, "welcome").await;

    let create_annotation = Request::builder()
        .method("POST")
        .uri("/api/notes/welcome/annotations")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::COOKIE, &reader_cookie)
        .header("x-csrf-token", &reader_csrf)
        .body(Body::from(
            json!({
                "start_offset": 0,
                "end_offset": 7,
                "quote": "Welcome",
                "comment": "admin managed comment",
                "visibility": "public"
            })
            .to_string(),
        ))
        .unwrap();
    let response = router.clone().oneshot(create_annotation).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let update_user = Request::builder()
        .method("POST")
        .uri("/admin/users/reader-one/update")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, &admin_cookie)
        .body(Body::from(format!(
            "username=reader-prime&new_password=PrimePass123&confirm_password=PrimePass123&_csrf={user_csrf}"
        )))
        .unwrap();
    let response = router.clone().oneshot(update_user).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(header::LOCATION).unwrap(),
        "/admin/users/reader-prime?status=updated"
    );

    let stale_session_request = Request::builder()
        .method("POST")
        .uri("/api/notes/welcome/annotations")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::COOKIE, &reader_cookie)
        .header("x-csrf-token", &reader_csrf)
        .body(Body::from(
            json!({
                "start_offset": 8,
                "end_offset": 10,
                "quote": "to",
                "comment": "should fail"
            })
            .to_string(),
        ))
        .unwrap();
    let response = router.clone().oneshot(stale_session_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let response = public_login(&router, "reader-one", "ReaderPass123", "/notes/welcome").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("用户名或密码错误"));

    let response = public_login(&router, "reader-prime", "PrimePass123", "/notes/welcome").await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);

    let public_annotations = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/notes/welcome/annotations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(public_annotations.status(), StatusCode::OK);
    let body = to_bytes(public_annotations.into_body(), usize::MAX)
        .await
        .unwrap();
    let listed: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(listed["annotations"].as_array().unwrap().len(), 1);
    assert_eq!(listed["annotations"][0]["username"], "reader-prime");

    let delete_user = Request::builder()
        .method("POST")
        .uri("/admin/users/reader-prime/delete")
        .header(header::COOKIE, &admin_cookie)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!("_csrf={user_csrf}")))
        .unwrap();
    let response = router.clone().oneshot(delete_user).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers().get(header::LOCATION).unwrap(),
        "/admin/users?status=deleted"
    );

    let users_page_after_delete = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin/users?status=deleted")
                .header(header::COOKIE, &admin_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(users_page_after_delete.status(), StatusCode::OK);
    let body = to_bytes(users_page_after_delete.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(!html.contains("reader-prime"));
    assert!(html.contains("用户已删除"));

    let annotations_after_delete = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/notes/welcome/annotations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(annotations_after_delete.into_body(), usize::MAX)
        .await
        .unwrap();
    let listed: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(listed["annotations"].as_array().unwrap().len(), 0);

    let response = public_login(&router, "reader-prime", "PrimePass123", "/notes/welcome").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("用户名或密码错误"));
}

#[tokio::test]
async fn public_user_auth_and_annotation_api_flow() {
    let (_temp, _state, router) = setup().await;

    let public_list_before_login = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/notes/welcome/annotations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(public_list_before_login.status(), StatusCode::OK);
    let body = to_bytes(public_list_before_login.into_body(), usize::MAX)
        .await
        .unwrap();
    let listed: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(listed["annotations"].as_array().unwrap().len(), 0);

    let unauthorized_create = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/notes/welcome/annotations")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "start_offset": 0,
                        "end_offset": 7,
                        "quote": "Welcome",
                        "comment": "公开评论",
                        "visibility": "public"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unauthorized_create.status(), StatusCode::UNAUTHORIZED);

    let user_cookie = public_register(&router, "alice", "ReaderPass123", "/notes/welcome").await;
    assert!(user_cookie.contains("m2w_user_session="));

    let (duplicate_cookie, duplicate_csrf) = account_pre_auth(&router, "/account").await;
    let duplicate_register = Request::builder()
        .method("POST")
        .uri("/account/register")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, duplicate_cookie)
        .body(Body::from(format!(
            "username=alice&password=ReaderPass123&next=/account&_csrf={duplicate_csrf}"
        )))
        .unwrap();
    let duplicate_response = router.clone().oneshot(duplicate_register).await.unwrap();
    assert_eq!(duplicate_response.status(), StatusCode::OK);
    let body = to_bytes(duplicate_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("该用户名已被注册"));

    let note_response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/notes/welcome")
                .header(header::COOKIE, &user_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(note_response.status(), StatusCode::OK);
    let body = to_bytes(note_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("当前已登录"));
    assert!(html.contains("alice"));
    assert!(html.contains("data-annotation-enabled=\"true\""));
    let user_csrf = csrf_from_note_html(&html);

    let create_request = Request::builder()
        .method("POST")
        .uri("/api/notes/welcome/annotations")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::COOKIE, &user_cookie)
        .header("x-csrf-token", &user_csrf)
        .body(Body::from(
            json!({
                "start_offset": 0,
                "end_offset": 7,
                "quote": "Welcome",
                "color": "#fde68a",
                "comment": "首段重点",
                "visibility": "public"
            })
            .to_string(),
        ))
        .unwrap();
    let create_response = router.clone().oneshot(create_request).await.unwrap();
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let body = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: Value = serde_json::from_slice(&body).unwrap();
    let annotation_id = created["id"].as_i64().unwrap();
    assert_eq!(created["comment"], "首段重点");
    assert_eq!(created["visibility"], "public");

    let list_response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/notes/welcome/annotations")
                .header(header::COOKIE, &user_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let body = to_bytes(list_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let listed: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(listed["annotations"].as_array().unwrap().len(), 1);

    let public_list = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/notes/welcome/annotations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(public_list.status(), StatusCode::OK);
    let body = to_bytes(public_list.into_body(), usize::MAX).await.unwrap();
    let listed: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(listed["annotations"].as_array().unwrap().len(), 1);
    assert_eq!(listed["annotations"][0]["visibility"], "public");

    let update_request = Request::builder()
        .method("PUT")
        .uri(format!("/api/annotations/{annotation_id}"))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::COOKIE, &user_cookie)
        .header("x-csrf-token", &user_csrf)
        .body(Body::from(
            json!({
                "color": "#bfdbfe",
                "comment": "更新后的评论",
                "visibility": "private"
            })
            .to_string(),
        ))
        .unwrap();
    let update_response = router.clone().oneshot(update_request).await.unwrap();
    assert_eq!(update_response.status(), StatusCode::OK);
    let body = to_bytes(update_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let updated: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["comment"], "更新后的评论");
    assert_eq!(updated["color"], "#bfdbfe");
    assert_eq!(updated["visibility"], "private");

    let public_list_after_private = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/notes/welcome/annotations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(public_list_after_private.status(), StatusCode::OK);
    let body = to_bytes(public_list_after_private.into_body(), usize::MAX)
        .await
        .unwrap();
    let listed: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(listed["annotations"].as_array().unwrap().len(), 0);

    let delete_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/annotations/{annotation_id}"))
                .header(header::COOKIE, &user_cookie)
                .header("x-csrf-token", &user_csrf)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    let list_after_delete = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/notes/welcome/annotations")
                .header(header::COOKIE, &user_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_after_delete.status(), StatusCode::OK);
    let body = to_bytes(list_after_delete.into_body(), usize::MAX)
        .await
        .unwrap();
    let listed: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(listed["annotations"].as_array().unwrap().len(), 0);

    let logout_request = Request::builder()
        .method("POST")
        .uri("/account/logout")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, &user_cookie)
        .body(Body::from(format!("next=/notes/welcome&_csrf={user_csrf}")))
        .unwrap();
    let logout_response = router.clone().oneshot(logout_request).await.unwrap();
    assert_eq!(logout_response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        logout_response.headers().get(header::LOCATION).unwrap(),
        "/notes/welcome"
    );

    let login_response = public_login(&router, "alice", "ReaderPass123", "/notes/welcome").await;
    assert_eq!(login_response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        login_response.headers().get(header::LOCATION).unwrap(),
        "/notes/welcome"
    );
}

#[tokio::test]
async fn danmaku_api_allows_public_list_and_requires_login_to_post() {
    let (_temp, _state, router) = setup().await;
    let video_key = "/assets/demo-video.mp4#0";

    let public_list = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/notes/welcome/danmaku?video_src=%2Fassets%2Fdemo-video.mp4%230")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(public_list.status(), StatusCode::OK);
    let body = to_bytes(public_list.into_body(), usize::MAX).await.unwrap();
    let listed: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(listed["danmaku"].as_array().unwrap().len(), 0);

    let unauthorized_create = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/notes/welcome/danmaku")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "video_src": video_key,
                        "time_ms": 1200,
                        "body": "第一条弹幕",
                        "color": "#22c55e"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unauthorized_create.status(), StatusCode::UNAUTHORIZED);

    let user_cookie =
        public_register(&router, "danmaku-user", "ReaderPass123", "/notes/welcome").await;
    let user_csrf = note_csrf(&router, &user_cookie, "welcome").await;

    let create_request = Request::builder()
        .method("POST")
        .uri("/api/notes/welcome/danmaku")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::COOKIE, &user_cookie)
        .header("x-csrf-token", &user_csrf)
        .body(Body::from(
            json!({
                "video_src": video_key,
                "time_ms": 1200,
                "body": "第一条弹幕",
                "color": "#22c55e"
            })
            .to_string(),
        ))
        .unwrap();
    let create_response = router.clone().oneshot(create_request).await.unwrap();
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let body = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(created["username"], "danmaku-user");
    assert_eq!(created["body"], "第一条弹幕");
    assert_eq!(created["color"], "#22c55e");
    assert_eq!(created["time_ms"], 1200);

    let list_response = router
        .oneshot(
            Request::builder()
                .uri("/api/notes/welcome/danmaku?video_src=%2Fassets%2Fdemo-video.mp4%230")
                .header(header::COOKIE, &user_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let body = to_bytes(list_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let listed: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(listed["danmaku"].as_array().unwrap().len(), 1);
    assert_eq!(listed["danmaku"][0]["video_src"], video_key);
}

#[tokio::test]
async fn rebuild_after_file_change_updates_site() {
    let (_temp, state, _router) = setup().await;

    let welcome = r#"---
title: Welcome to markdown2web
slug: welcome
summary: Updated summary
tags: [intro]
status: published
---
# Welcome to markdown2web

This content changed after rebuild.
"#;
    filesystem::write_note(&state.config, "welcome", welcome).unwrap();
    state
        .build_service
        .rebuild("integration rebuild")
        .await
        .unwrap();

    let site = state.site.read().await.clone();
    let note = site.note("welcome").unwrap();
    assert!(
        note.raw_markdown
            .contains("This content changed after rebuild")
    );
}

#[tokio::test]
async fn note_page_emits_math_markers_and_highlighted_code() {
    let (_temp, state, router) = setup().await;

    let note = r#"---
title: Render Test
slug: render-test
summary: math and code
tags: [render]
status: published
---
# Render Test

$$\int_0^1 x^2 dx = \frac{1}{3}$$

```rust
fn main() {
    println!("hi");
}
```
"#;
    filesystem::write_note(&state.config, "render-test", note).unwrap();
    state
        .build_service
        .rebuild("render behavior")
        .await
        .unwrap();

    let response = router
        .oneshot(
            Request::builder()
                .uri("/notes/render-test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("data-math-style=\"display\""));
    assert!(html.contains("background-color:") || html.contains("<span style="));
}

#[tokio::test]
async fn draft_notes_are_hidden_from_public_routes() {
    let (_temp, state, router) = setup().await;

    let draft = r#"---
title: Secret Draft
slug: secret-draft
summary: Hidden
tags: [private]
status: draft
---
# Secret Draft

Not public.
"#;
    filesystem::write_note(&state.config, "secret-draft", draft).unwrap();
    state.build_service.rebuild("draft test").await.unwrap();

    let response = router
        .oneshot(
            Request::builder()
                .uri("/notes/secret-draft")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn cursor_beacon_replaces_card_local_glow() {
    let base = fs::read_to_string("templates/base.html").unwrap();
    let css = fs::read_to_string("static/css/app.css").unwrap();
    let js = fs::read_to_string("static/js/site.js").unwrap();
    assert!(base.contains("cursor-beacon"));
    assert!(css.contains(".cursor-beacon"));
    assert!(js.contains("wireCursorBeacon"));
    assert!(!css.contains(".interactive-card::after"));
}

#[test]
fn cursor_beacon_is_ring_like_not_filled_disc() {
    let css = fs::read_to_string("static/css/app.css").unwrap();
    let start = css.find(".cursor-beacon {").unwrap();
    let end = css.find(".cursor-beacon::before {").unwrap();
    let beacon_block = &css[start..end];

    assert!(beacon_block.contains("background: transparent;"));
    assert!(!beacon_block.contains("backdrop-filter"));
    assert!(css.contains("margin-left: 12px;"));
    assert!(css.contains("content: none;"));
    assert!(css.contains(".cursor-beacon-core {\n  display: none;"));
}

#[test]
fn code_copy_button_has_feedback_and_fallback() {
    let js = fs::read_to_string("static/js/site.js").unwrap();
    let css = fs::read_to_string("static/css/app.css").unwrap();

    assert!(js.contains("navigator.clipboard?.writeText"));
    assert!(js.contains("document.execCommand('copy')"));
    assert!(js.contains("Clipboard API copy failed; falling back"));
    assert!(js.contains("setCopyButtonFeedback(button, 'Copied', 'copied')"));
    assert!(js.contains("setCopyButtonFeedback(button, 'Failed', 'copy-failed')"));
    assert!(js.contains("button.setAttribute('aria-live', 'polite')"));
    assert!(css.contains(".copy-code-button.copy-failed"));
}

#[test]
fn note_sidebar_spacing_and_toc_hover_styles_exist() {
    let css = fs::read_to_string("static/css/app.css").unwrap();
    assert!(css.contains(".sidebar-title-row {\n  margin-bottom: 14px;"));
    assert!(css.contains(".toc-list a:hover"));
    assert!(css.contains("font-size: 1.03rem;"));
    assert!(css.contains("font-weight: 700;"));
}

#[test]
fn note_layout_places_sidebar_left_article_center_and_rail_right() {
    let css = fs::read_to_string("static/css/app.css").unwrap();
    assert!(css.contains(".note-layout {\n  display: grid;"));
    assert!(css.contains("grid-template-columns: 260px minmax(0, 1fr) 300px;"));
    assert!(css.contains("grid-template-areas: \"sidebar article rail\";"));
    assert!(css.contains(".note-article {\n  grid-area: article;"));
    assert!(css.contains(".note-sidebar {\n  grid-area: sidebar;"));
    assert!(css.contains(".annotation-rail { grid-area: rail;"));
    assert!(css.contains(".note-layout {\n  display: grid;"));
    assert!(css.contains("min-width: 0;"));
    assert!(css.contains("overflow: visible;"));
    assert!(css.contains("touch-action: pan-y pinch-zoom;"));
    assert!(css.contains(".note-article.interactive-card:hover"));
    assert!(css.contains("transform: none;"));
    assert!(
        css.contains("grid-template-areas:\n      \"article\"\n      \"sidebar\"\n      \"rail\";")
    );
}

#[test]
fn note_article_does_not_become_touch_scroll_container() {
    let css = fs::read_to_string("static/css/app.css").unwrap();
    let note_article_start = css.find(".note-article {\n").unwrap();
    let note_article_block = &css
        [note_article_start..css[note_article_start..].find("}\n").unwrap() + note_article_start];

    assert!(note_article_block.contains("overflow: visible;"));
    assert!(note_article_block.contains("touch-action: pan-y pinch-zoom;"));
    assert!(note_article_block.contains("transform: none;"));
    assert!(!note_article_block.contains("overflow-x: hidden;"));
    assert!(!note_article_block.contains("overflow-x: clip;"));
    assert!(!note_article_block.contains("overflow-y: auto;"));
    assert!(!note_article_block.contains("overflow: hidden;"));
    assert!(!note_article_block.contains("transition:\n    transform"));
    assert!(
        css.contains(".note-article.reveal-on-scroll,\n.note-article.reveal-on-scroll.is-visible")
    );
    assert!(css.contains(".note-article.note-card:hover"));
    assert!(css.contains(".note-article::before,\n.note-article:hover::before"));
    assert!(css.contains(".note-article-body {\n  position: relative;"));
    assert!(css.contains(".note-article :where(h1, h2, h3, h4, h5, h6, p, li, blockquote, figure, img, pre, code, table)"));
    assert!(css.contains("@media (hover: none), (pointer: coarse)"));
    assert!(css.contains(".interactive-card-subtle:hover {\n    transform: none;"));

    let js = fs::read_to_string("static/js/site.js").unwrap();
    assert!(js.contains(".filter((element) => !element.hasAttribute('data-note-article'))"));
    assert!(js.contains("let touchMoved = false;"));
    assert!(js.contains("const touchMoveThreshold = 8;"));
    assert!(js.contains("root.addEventListener('touchmove'"));
    assert!(js.contains("if (touchMoved) return;"));
    assert!(js.contains("{ passive: true }"));
}

#[test]
fn mascot_wiring_exists() {
    let home = fs::read_to_string("templates/home.html").unwrap();
    let css = fs::read_to_string("static/css/app.css").unwrap();
    let js = fs::read_to_string("static/js/site.js").unwrap();

    // HTML 模板包含吉祥物元素
    assert!(home.contains("hero-mascot-panel"));
    assert!(home.contains("mascot-stage"));
    assert!(home.contains("data-mascot"));
    assert!(home.contains("data-pupil"));
    assert!(home.contains("data-mouth"));
    assert!(home.contains("data-mascot-label"));
    assert!(home.contains("mascot-eye mascot-eye-l"));
    assert!(home.contains("mascot-blush"));

    // CSS 包含吉祥物样式
    assert!(css.contains(".hero-mascot-panel"));
    assert!(css.contains("--mascot-size"));
    assert!(css.contains(".mascot-stage"));
    assert!(css.contains(".mascot-face"));
    assert!(css.contains(".mascot-eye"));
    assert!(css.contains(".mascot-pupil"));
    assert!(css.contains(".mascot-mouth"));
    assert!(css.contains(".mascot-blush"));
    assert!(css.contains(".mascot-ear"));
    assert!(css.contains(".mascot-label"));

    // JS 包含吉祥物逻辑
    assert!(js.contains("wireMascot"));
    assert!(js.contains("data-pupil"));
    assert!(js.contains("data-mouth"));
    assert!(js.contains("data-mascot-label"));
    assert!(js.contains("switchExpression"));
    assert!(js.contains("trackEyes"));

    // 确认旧的 orb 元素已移除
    assert!(!home.contains("hero-orb-panel"));
    assert!(!home.contains("data-orbital-focus"));
    assert!(!js.contains("wireOrbitalFocus"));
    assert!(!css.contains(".hero-orb-panel"));
}

#[test]
fn admin_user_management_wiring_exists() {
    let dashboard = fs::read_to_string("templates/admin/dashboard.html").unwrap();
    let users = fs::read_to_string("templates/admin/users.html").unwrap();
    let user_edit = fs::read_to_string("templates/admin/user_edit.html").unwrap();
    let css = fs::read_to_string("static/css/app.css").unwrap();

    assert!(dashboard.contains("href=\"/admin/users\""));
    assert!(dashboard.contains("用户管理"));

    assert!(users.contains("action=\"/admin/users\""));
    assert!(users.contains("已注册用户"));
    assert!(users.contains("managed_user.route_key"));

    assert!(user_edit.contains("action=\"/admin/users/{{ managed_user.route_key }}/update\""));
    assert!(user_edit.contains("action=\"/admin/users/{{ managed_user.route_key }}/delete\""));
    assert!(user_edit.contains("删除用户"));

    assert!(css.contains(".admin-users-grid"));
    assert!(css.contains(".admin-user-row"));
    assert!(css.contains(".admin-danger-button"));
}

#[test]
fn admin_asset_upload_accepts_audio_and_video() {
    let dashboard = fs::read_to_string("templates/admin/dashboard.html").unwrap();
    let admin = fs::read_to_string("src/web/admin.rs").unwrap();

    assert!(dashboard.contains(".mp3"));
    assert!(dashboard.contains(".mp4"));
    assert!(!dashboard.contains(".svg"));
    assert!(admin.contains("\"mp3\""));
    assert!(admin.contains("\"mp4\""));
    assert!(!admin.contains("\"svg\""));
    assert!(admin.contains("webp, mp3, mp4, pdf"));
}

#[test]
fn turnstile_runs_only_after_auth_form_submit() {
    let base = fs::read_to_string("templates/base.html").unwrap();
    let admin_login = fs::read_to_string("templates/admin/login.html").unwrap();
    let account = fs::read_to_string("templates/account.html").unwrap();
    let css = fs::read_to_string("static/css/app.css").unwrap();
    let js = fs::read_to_string("static/js/site.js").unwrap();

    assert!(!base.contains("challenges.cloudflare.com/turnstile"));
    assert!(!admin_login.contains("class=\"cf-turnstile\""));
    assert!(!account.contains("class=\"cf-turnstile\""));

    assert!(admin_login.contains("data-turnstile-form"));
    assert!(admin_login.contains("data-turnstile-response"));
    assert!(admin_login.contains("data-turnstile-lazy"));
    assert!(account.matches("data-turnstile-form").count() >= 2);
    assert!(account.matches("data-turnstile-lazy").count() >= 2);

    assert!(js.contains("function loadTurnstileScript()"));
    assert!(js.contains("api.js?render=explicit"));
    assert!(js.contains("await loadTurnstileScript();"));
    assert!(js.contains("window.turnstile.execute(widgetId);"));
    assert!(js.contains("'response-field': false"));
    assert!(js.contains("removeDuplicateResponseFields"));
    assert!(js.contains(r#"input[name="cf-turnstile-response"]:not([data-turnstile-response])"#));
    assert!(css.contains(".turnstile-lazy"));
}

#[test]
fn annotation_wiring_exists() {
    let base = fs::read_to_string("templates/base.html").unwrap();
    let note = fs::read_to_string("templates/note.html").unwrap();
    let account = fs::read_to_string("templates/account.html").unwrap();
    let css = fs::read_to_string("static/css/app.css").unwrap();
    let js = fs::read_to_string("static/js/site.js").unwrap();

    assert!(base.contains("href=\"/account\""));

    assert!(note.contains("data-note-article"));
    assert!(note.contains("data-annotation-root"));
    assert!(note.contains("data-annotation-toolbar"));
    assert!(note.contains("data-annotation-comment-lane"));
    assert!(note.contains("data-viewer-username"));
    assert!(note.contains("note-page-shell"));
    assert!(note.contains("annotation-auth-card"));
    assert!(note.contains("/account?next=/notes/{{ note.slug }}"));
    assert!(note.contains("data-annotation-modal"));
    assert!(note.contains("data-annotation-comment-input"));
    assert!(note.contains("data-annotation-comment-save"));
    assert!(note.contains("annotation-modal"));
    assert!(note.contains("data-annotation-visibility"));
    assert!(note.contains(">私密<"));
    assert!(note.contains(">公开<"));

    assert!(account.contains("action=\"/account/login\""));
    assert!(account.contains("action=\"/account/register\""));

    assert!(css.contains(".annotation-toolbar"));
    assert!(css.contains(".shell.note-page-shell"));
    assert!(css.contains(".note-annotation"));
    assert!(css.contains(".annotation-comment-card"));
    assert!(css.contains(".annotation-rail"));
    assert!(css.contains(".audio-icon-pause rect"));
    assert!(css.contains(".audio-play-btn.is-playing .audio-icon-play"));
    assert!(css.contains(".audio-play-btn.is-playing .audio-icon-pause"));
    assert!(css.contains(".video-player"));
    assert!(css.contains(".video-player-media"));
    assert!(css.contains(".responsive-image"));
    assert!(css.contains(".video-load-button"));
    assert!(css.contains(".video-player.is-loaded .video-load-button"));
    assert!(css.contains(".video-controls"));
    assert!(css.contains(".video-control-button"));
    assert!(css.contains(".video-progress-fill"));
    assert!(css.contains(".video-time"));
    assert!(css.contains(".video-volume-control"));
    assert!(css.contains(".video-fullscreen-button svg"));
    assert!(css.contains("rgba(255, 255, 255, 0.94)"));
    assert!(css.contains("#00a1d6"));
    assert!(css.contains(".video-danmaku-layer"));
    assert!(css.contains(".video-danmaku-item"));
    assert!(css.contains("@keyframes danmaku-fly"));
    assert!(css.contains(".video-speed-select"));
    assert!(css.contains(".video-danmaku-size-select"));
    assert!(css.contains("--danmaku-font-size, 1.25rem"));
    assert!(css.contains(".video-danmaku-form"));
    assert!(css.contains(".video-danmaku-color"));
    assert!(css.contains(".video-danmaku-login"));
    assert!(css.contains(".video-load-button:hover"));
    assert!(css.contains(".video-load-button:active"));
    assert!(css.contains(".video-load-button::before"));
    assert!(css.contains("transform: translate(-50%, -50%);"));
    assert!(css.contains(".note-article {"));
    assert!(css.contains("overflow: visible;"));
    assert!(css.contains(".note-article.interactive-card:hover"));
    assert!(css.contains(".prose :where(img, video, iframe, figure, table, pre)"));
    assert!(css.contains(".video-player-frame"));
    assert!(css.contains("contain: inline-size layout paint;"));
    assert!(css.contains("position: absolute;"));
    assert!(css.contains("inset: 0;"));
    assert!(css.contains("max-inline-size: 100%;"));
    assert!(css.contains("min-inline-size: 0;"));
    assert!(css.contains("object-fit: contain;"));
    assert!(!css.contains(".video-label"));
    assert!(js.contains("const renderAnnotations = () => {"));
    assert!(js.contains("renderMath();"));
    assert!(js.contains("wireAudioPlayers();"));
    assert!(js.contains("wireVideoPlayers();"));
    assert!(js.contains("element.hasAttribute('data-static-button')"));
    assert!(js.contains("source.setAttribute('src'"));
    assert!(js.contains("video.load();"));
    assert!(js.contains("const updateControls = () => {"));
    assert!(js.contains("const updateTime = () => {"));
    assert!(js.contains("data-video-toggle"));
    assert!(js.contains("requestFullscreen"));
    assert!(js.contains("is-controls-visible"));
    assert!(js.contains("window.setTimeout"));
    assert!(js.contains("container.matches(':focus-within')"));
    assert!(js.contains("/danmaku"));
    assert!(js.contains("data-video-speed"));
    assert!(js.contains("data-video-danmaku-size"));
    assert!(js.contains("--danmaku-font-size"));
    assert!(js.contains("data-video-danmaku-form"));
    assert!(js.contains("data-video-danmaku-color"));
    assert!(js.contains("showDanmaku"));
    assert!(js.contains("syncDanmaku(true);"));
    assert!(js.contains("video.addEventListener('seeking'"));
    assert!(js.contains("video.addEventListener('seeked'"));
    assert!(js.contains("item.time_ms >= startMs && item.time_ms <= endMs"));
    assert!(js.contains("video.playbackRate"));
    assert!(js.contains("const setPlaybackUi = () => {"));
    assert!(js.contains("const updateTimeDisplay = () => {"));
    assert!(js.contains("playBtn.classList.toggle('is-playing', isPlaying);"));
    assert!(js.contains("timeDisplay.textContent = `${current}/${total}`;"));
    assert!(css.contains(".annotation-visibility-select"));
    assert!(css.contains(".annotation-comment-visibility"));
    assert!(css.contains(".annotation-modal"));
    assert!(css.contains("body.annotation-modal-open"));

    assert!(js.contains("wireNoteAnnotations"));
    assert!(js.contains("data-annotation-highlight"));
    assert!(js.contains("data-annotation-visibility"));
    assert!(js.contains("visibilitySelect.value"));
    assert!(js.contains("addEventListener('mousedown'"));
    assert!(js.contains("addEventListener('touchmove'"));
    assert!(js.contains("if (touchMoved) return;"));
    assert!(js.contains("addEventListener('contextmenu'"));
    assert!(js.contains("openOwnedAnnotationPanel"));
    assert!(js.contains("annotationFromEvent"));
    assert!(js.contains("button !== 2"));
    assert!(js.contains("openCommentModal"));
    assert!(js.contains("data-annotation-comment-save"));
    assert!(!js.contains("window.prompt"));
    assert!(js.contains("/api/notes/"));
    assert!(!js.contains("m2w_user_session"));
}
