use std::sync::LazyLock;

use serde::Deserialize;

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

#[derive(Deserialize)]
struct TurnstileResponse {
    success: bool,
    #[serde(rename = "error-codes")]
    error_codes: Option<Vec<String>>,
}

/// 验证 Cloudflare Turnstile token。
/// 如果 secret_key 为空（未配置环境变量），直接返回 true 跳过验证。
pub async fn verify_turnstile(
    token: &str,
    secret_key: &str,
    remote_ip: Option<&str>,
) -> Result<bool, String> {
    if secret_key.is_empty() {
        return Ok(true);
    }

    if token.is_empty() {
        return Ok(false);
    }

    let mut form = vec![
        ("secret", secret_key.to_string()),
        ("response", token.to_string()),
    ];
    if let Some(ip) = remote_ip {
        form.push(("remoteip", ip.to_string()));
    }

    let resp = HTTP_CLIENT
        .post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
        .form(&form)
        .send()
        .await
        .map_err(|e| format!("Turnstile request failed: {e}"))?;

    let result: TurnstileResponse = resp
        .json()
        .await
        .map_err(|e| format!("Turnstile parse failed: {e}"))?;

    if !result.success {
        let errors = result
            .error_codes
            .map(|codes| codes.join(", "))
            .unwrap_or_else(|| "unknown".into());
        return Err(format!("Turnstile verification failed: {errors}"));
    }

    Ok(true)
}
