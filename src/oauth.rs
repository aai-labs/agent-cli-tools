use reqwest::Client;
use serde_json::Value;

use crate::{config::Profile, error::AppError};

/// Resolve an access token for the given profile.
///
/// Priority:
///   1. If refresh_token + client_id + client_secret are all set → exchange for a fresh token
///   2. If a stored access token (token / api_token) is present → use it directly
///   3. Otherwise → error
pub(crate) async fn resolve_token(
    profile: &Profile,
    client: &Client,
    service: &'static str,
    operation: &'static str,
) -> Result<String, AppError> {
    if matches!(profile.auth_type.as_deref(), Some("none")) {
        return Ok(String::new());
    }

    if let (Some(refresh_token), Some(client_id), Some(client_secret)) = (
        profile.refresh_token.as_deref(),
        profile.client_id.as_deref(),
        profile.client_secret.as_deref(),
    ) {
        return exchange(client, profile, refresh_token, client_id, client_secret).await;
    }

    if let Some(token) = profile.token.as_deref().or(profile.api_token.as_deref()) {
        return Ok(token.to_string());
    }

    Err(AppError::auth(
        service,
        operation,
        "profile has no access token or refresh credentials; \
         set token_secret, or set refresh_token_secret + client_id + client_secret_secret",
    ))
}

async fn exchange(
    client: &Client,
    profile: &Profile,
    refresh_token: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<String, AppError> {
    let endpoint = token_endpoint(profile);
    let body = format!(
        "grant_type=refresh_token&refresh_token={}&client_id={}&client_secret={}",
        urlencoding::encode(refresh_token),
        urlencoding::encode(client_id),
        urlencoding::encode(client_secret),
    );
    let resp = client
        .post(endpoint)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|e| AppError::internal("oauth", "refresh", e.to_string()))?;
    let status = resp.status();
    let body: Value = resp
        .json()
        .await
        .map_err(|e| AppError::internal("oauth", "refresh", e.to_string()))?;
    if !status.is_success() {
        return Err(AppError::auth(
            "oauth",
            "refresh",
            format!("token refresh failed (HTTP {}): {body}", status.as_u16()),
        ));
    }
    body.get("access_token")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| AppError::auth("oauth", "refresh", "token response missing access_token"))
}

fn token_endpoint(profile: &Profile) -> &'static str {
    match profile.auth_type.as_deref() {
        Some("zoho_oauth" | "zoho-oauth") => "https://accounts.zoho.com/oauth/v2/token",
        _ => "https://oauth2.googleapis.com/token",
    }
}
