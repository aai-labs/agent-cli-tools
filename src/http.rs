use base64::{engine::general_purpose, Engine as _};
use reqwest::{Client, Method, RequestBuilder};
use serde_json::Value;

use crate::{config::Profile, error::AppError};

pub struct ApiClient {
    client: Client,
}

impl ApiClient {
    pub fn new() -> Result<Self, AppError> {
        let client = Client::builder()
            .user_agent("aai-cli/0.1")
            .build()
            .map_err(|err| AppError::internal("http", "client", err.to_string()))?;
        Ok(Self { client })
    }

    pub async fn request(
        &self,
        service: &'static str,
        operation: &'static str,
        profile: &Profile,
        method: Method,
        url: String,
        body: Option<Value>,
    ) -> Result<Value, AppError> {
        let mut request = self.client.request(method, &url);
        request = apply_auth(request, service, operation, profile)?;
        request = request.header("Accept", "application/json");
        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await.map_err(|err| {
            AppError::internal(service, operation, format!("request failed: {err}"))
        })?;
        let status = response.status();
        let text = response.text().await.map_err(|err| {
            AppError::internal(
                service,
                operation,
                format!("failed to read response: {err}"),
            )
        })?;
        let parsed = if text.trim().is_empty() {
            Value::Object(serde_json::Map::new())
        } else {
            serde_json::from_str(&text).unwrap_or_else(|_| Value::String(text.clone()))
        };

        if status.is_success() {
            Ok(parsed)
        } else {
            Err(AppError::api(
                service,
                operation,
                status,
                format!("provider returned HTTP {}", status.as_u16()),
                Some(parsed),
            ))
        }
    }

    pub async fn download(
        &self,
        service: &'static str,
        operation: &'static str,
        profile: &Profile,
        url: String,
    ) -> Result<Vec<u8>, AppError> {
        let mut request = self.client.request(Method::GET, &url);
        request = apply_auth(request, service, operation, profile)?;
        let accept = match service {
            "github" => "application/json",
            "bitbucket" => "*/*",
            _ => "*/*",
        };
        request = request.header("Accept", accept);

        let response = request.send().await.map_err(|err| {
            AppError::internal(service, operation, format!("request failed: {err}"))
        })?;
        let status = response.status();
        let bytes = response.bytes().await.map_err(|err| {
            AppError::internal(
                service,
                operation,
                format!("failed to read response: {err}"),
            )
        })?;

        if status.is_success() {
            Ok(bytes.to_vec())
        } else {
            let details = std::str::from_utf8(&bytes)
                .ok()
                .and_then(|text| serde_json::from_str(text).ok())
                .or_else(|| {
                    Some(Value::String(
                        String::from_utf8_lossy(&bytes).chars().take(4096).collect(),
                    ))
                });
            Err(AppError::api(
                service,
                operation,
                status,
                format!("provider returned HTTP {}", status.as_u16()),
                details,
            ))
        }
    }

    pub async fn upload(
        &self,
        service: &'static str,
        operation: &'static str,
        profile: &Profile,
        url: String,
        file_path: &str,
        comment: Option<&str>,
    ) -> Result<Value, AppError> {
        let file_bytes = std::fs::read(file_path).map_err(|e| {
            AppError::internal(service, operation, format!("failed to read file: {e}"))
        })?;
        let filename = std::path::Path::new(file_path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "file".to_string());

        let boundary = "----AaiCliBoundary7b2c4e9f1d3a6b8c";
        let mut body: Vec<u8> = Vec::new();

        // file part
        let file_header = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\nContent-Type: application/octet-stream\r\n\r\n"
        );
        body.extend_from_slice(file_header.as_bytes());
        body.extend_from_slice(&file_bytes);
        body.extend_from_slice(b"\r\n");

        // optional comment part
        if let Some(c) = comment {
            let comment_header =
                format!("--{boundary}\r\nContent-Disposition: form-data; name=\"comment\"\r\n\r\n");
            body.extend_from_slice(comment_header.as_bytes());
            body.extend_from_slice(c.as_bytes());
            body.extend_from_slice(b"\r\n");
        }

        // closing boundary
        body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

        let content_type = format!("multipart/form-data; boundary={boundary}");
        let mut request = self
            .client
            .post(&url)
            .body(body)
            .header("Content-Type", content_type);
        request = apply_auth(request, service, operation, profile)?;
        request = request.header("X-Atlassian-Token", "no-check");

        let response = request
            .send()
            .await
            .map_err(|e| AppError::internal(service, operation, format!("request failed: {e}")))?;
        let status = response.status();
        let text = response.text().await.map_err(|e| {
            AppError::internal(service, operation, format!("failed to read response: {e}"))
        })?;
        let parsed = if text.trim().is_empty() {
            Value::Object(serde_json::Map::new())
        } else {
            serde_json::from_str(&text).unwrap_or(Value::String(text))
        };
        if status.is_success() {
            Ok(parsed)
        } else {
            Err(AppError::api(
                service,
                operation,
                status,
                format!("provider returned HTTP {}", status.as_u16()),
                Some(parsed),
            ))
        }
    }
}

fn apply_auth(
    request: RequestBuilder,
    service: &'static str,
    operation: &'static str,
    profile: &Profile,
) -> Result<RequestBuilder, AppError> {
    let auth_type = profile.auth_type.as_deref().unwrap_or("bearer_token");
    match auth_type {
        "basic_api_token" | "basic" => {
            let username = profile
                .email
                .as_deref()
                .or(profile.username.as_deref())
                .ok_or_else(|| {
                    AppError::auth(service, operation, "profile is missing email or username")
                })?;
            let token = profile
                .api_token
                .as_deref()
                .or(profile.token.as_deref())
                .ok_or_else(|| {
                    AppError::auth(service, operation, "profile is missing api_token or token")
                })?;
            Ok(request.header(
                "Authorization",
                format!(
                    "Basic {}",
                    general_purpose::STANDARD.encode(format!("{username}:{token}"))
                ),
            ))
        }
        "none" => Ok(request),
        "zoho_oauth" | "zoho-oauth" => {
            let token = profile
                .token
                .as_deref()
                .or(profile.api_token.as_deref())
                .ok_or_else(|| AppError::auth(service, operation, "profile is missing token"))?;
            Ok(request.header("Authorization", format!("Zoho-oauthtoken {token}")))
        }
        _ => {
            let token = profile
                .token
                .as_deref()
                .or(profile.api_token.as_deref())
                .ok_or_else(|| AppError::auth(service, operation, "profile is missing token"))?;
            Ok(request.bearer_auth(token))
        }
    }
}
