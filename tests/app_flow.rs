use std::fs;
use std::sync::Arc;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use markdown2web::{
    app,
    config::AppConfig,
    store::{filesystem, sqlite::AppDatabase},
};
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
        admin_password: "Pcsensor1121@".into(),
        watch_enabled: false,
        upload_limit_mb: 10,
    }
}

async fn setup() -> (TempDir, app::AppState, axum::Router) {
    let temp = TempDir::new().unwrap();
    let config = test_config(&temp);
    config.ensure_directories().unwrap();
    let db = Arc::new(AppDatabase::open(&config.database_path()).unwrap());
    db.initialize(&config.admin_username, &config.admin_password)
        .unwrap();
    let state = app::AppState::bootstrap(config, db).await.unwrap();
    let router = app::build_router(state.clone());
    (temp, state, router)
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
    assert!(html.contains("hero-panel panel interactive-card hero-particle-panel"));
    assert!(html.contains("data-particle-field"));
    assert!(html.contains("hero-particle-canvas"));
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

    let login_request = Request::builder()
        .method("POST")
        .uri("/admin/login")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from("username=admin&password=Pcsensor1121@"))
        .unwrap();
    let response = router.clone().oneshot(login_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let save_request = Request::builder()
        .method("POST")
        .uri("/admin/notes/save")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .header(header::COOKIE, cookie)
        .body(Body::from("title=Integration%20Note&summary=Saved%20from%20test&tags=testing,axum&status=published&aliases=&body=%23%20Integration%20Note%0A%0AHello%20from%20test"))
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
fn note_sidebar_spacing_and_toc_hover_styles_exist() {
    let css = fs::read_to_string("static/css/app.css").unwrap();
    assert!(css.contains(".sidebar-title-row {\n  margin-bottom: 14px;"));
    assert!(css.contains(".toc-list a:hover"));
    assert!(css.contains("font-size: 1.03rem;"));
    assert!(css.contains("font-weight: 700;"));
}

#[test]
fn hero_particle_stage_wiring_exists() {
    let home = fs::read_to_string("templates/home.html").unwrap();
    let css = fs::read_to_string("static/css/app.css").unwrap();
    let js = fs::read_to_string("static/js/site.js").unwrap();

    assert!(home.contains("hero-particle-stage\" data-particle-field"));
    assert!(home.contains("hero-particle-canvas"));
    assert!(css.contains(".hero-particle-stage"));
    assert!(css.contains("min-height: 460px;"));
    assert!(css.contains("z-index: 0;"));
    assert!(css.contains("inset: -8% -12% -8% 0;"));
    assert!(js.contains("wireHeroParticles"));
    assert!(js.contains("closest('.hero-particle-panel')"));
    assert!(js.contains("state.clickImpulse"));
    assert!(js.contains("scheduleAmbientBurst"));
    assert!(js.contains("randomRange(5.2, 8.2)"));
    assert!(js.contains("* 0.0085"));
}
