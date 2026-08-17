use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::{engine::general_purpose, Engine as _};
use reqwest::{Client, Method, RequestBuilder};
use serde_json::Value;

use crate::{config::Profile, error::AppError};

pub(crate) fn multipart_boundary() -> String {
fn multipart_boundary() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("----AaiCliBoundary{nanos:x}{n:x}")
}

pub struct ApiClient {
    client: Client,
    no_redirect_client: Client,
}

/// A request whose body is already-encoded bytes rather than a JSON value.
///
/// Providers that take pre-encoded payloads — multipart/related upload bodies, raw
/// media bytes — need a content type and occasionally extra headers alongside the
/// body, so they travel together instead of as five positional arguments.
pub struct BytesRequest {
    pub method: Method,
    pub url: String,
    pub content_type: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// A parsed response plus the `Location` header.
///
/// Upload protocols that hand back a session URI put it in `Location` and leave the
/// body empty, so the header cannot be dropped the way `request` drops it.
pub struct RawResponse {
    pub body: Value,
    pub location: Option<String>,
}

impl ApiClient {
    pub fn new() -> Result<Self, AppError> {
        let client = Client::builder()
            .user_agent("aai-cli/0.1")
            .build()
            .map_err(|err| AppError::internal("http", "client", err.to_string()))?;
        let no_redirect_client = Client::builder()
            .user_agent("aai-cli/0.1")
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|err| AppError::internal("http", "client", err.to_string()))?;
        Ok(Self {
            client,
            no_redirect_client,
        })
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
        Self::request_with(&self.client, service, operation, profile, method, url, body)
            .await
            .map(|(value, _)| value)
    }

    pub async fn request_no_redirect(
        &self,
        service: &'static str,
        operation: &'static str,
        profile: &Profile,
        method: Method,
        url: String,
        body: Option<Value>,
    ) -> Result<Value, AppError> {
        let (value, next_url) = Self::request_with(
            &self.no_redirect_client,
            service,
            operation,
            profile,
            method,
            url,
            body,
        )
        .await?;
        Ok(attach_provider_next_url(value, next_url))
    }

    async fn request_with(
        client: &Client,
        service: &'static str,
        operation: &'static str,
        profile: &Profile,
        method: Method,
        url: String,
        body: Option<Value>,
    ) -> Result<(Value, Option<String>), AppError> {
        let token = crate::oauth::resolve_token(profile, client, service, operation).await?;
        let effective = crate::config::Profile {
            token: Some(token),
            ..profile.clone()
        };
        let mut request = client.request(method, &url);
        request = apply_auth(request, service, operation, &effective)?;
        request = request.header("Accept", "application/json");
        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await.map_err(|err| {
            AppError::internal(service, operation, format!("request failed: {err}"))
        })?;
        let status = response.status();
        let next_url = response
            .headers()
            .get(reqwest::header::LINK)
            .and_then(|value| value.to_str().ok())
            .and_then(link_next_url);
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
            Ok((parsed, next_url))
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
        let accept = match service {
            "github" => "application/json",
            "bitbucket" => "*/*",
            _ => "*/*",
        };
        self.download_with_accept(service, operation, profile, url, accept)
            .await
    }

    pub async fn download_with_accept(
        &self,
        service: &'static str,
        operation: &'static str,
        profile: &Profile,
        url: String,
        accept: &str,
    ) -> Result<Vec<u8>, AppError> {
        let token = crate::oauth::resolve_token(profile, &self.client, service, operation).await?;
        let effective = crate::config::Profile {
            token: Some(token),
            ..profile.clone()
        };
        let mut request = self.client.request(Method::GET, &url);
        request = apply_auth(request, service, operation, &effective)?;
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

    /// Send an already-encoded byte body and parse the JSON reply.
    ///
    /// `upload` below builds one specific multipart/form-data shape for Atlassian.
    /// This is the general form: the caller owns the encoding, this owns auth,
    /// execution, and error mapping.
    pub async fn request_bytes(
        &self,
        service: &'static str,
        operation: &'static str,
        profile: &Profile,
        request: BytesRequest,
    ) -> Result<RawResponse, AppError> {
        let token = crate::oauth::resolve_token(profile, &self.client, service, operation).await?;
        let effective = crate::config::Profile {
            token: Some(token),
            ..profile.clone()
        };
        let mut builder = self
            .client
            .request(request.method, &request.url)
            .header("Content-Type", request.content_type)
            .header("Accept", "application/json")
            .body(request.body);
        for (name, value) in request.headers {
            builder = builder.header(name, value);
        }
        builder = apply_auth(builder, service, operation, &effective)?;

        let response = builder.send().await.map_err(|err| {
            AppError::internal(service, operation, format!("request failed: {err}"))
        })?;
        let status = response.status();
        let location = response
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|value| value.to_str().ok())
            .map(str::to_string);
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
            Ok(RawResponse {
                body: parsed,
                location,
            })
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

        let boundary = multipart_boundary();
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

        let token = crate::oauth::resolve_token(profile, &self.client, service, operation).await?;
        let effective = crate::config::Profile {
            token: Some(token),
            ..profile.clone()
        };
        let content_type = format!("multipart/form-data; boundary={boundary}");
        let mut request = self
            .client
            .post(&url)
            .body(body)
            .header("Content-Type", content_type);
        request = apply_auth(request, service, operation, &effective)?;
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

fn link_next_url(link: &str) -> Option<String> {
    link.split(',').find_map(|part| {
        let part = part.trim();
        if !part.contains("rel=\"next\"") && !part.contains("rel=next") {
            return None;
        }
        let start = part.find('<')? + 1;
        let end = part[start..].find('>')? + start;
        Some(part[start..end].to_string())
    })
}

fn attach_provider_next_url(value: Value, next_url: Option<String>) -> Value {
    let Some(next_url) = next_url else {
        return value;
    };
    match value {
        Value::Object(mut object) => {
            object.insert(
                "_aai_provider_next_url".to_string(),
                Value::String(next_url),
            );
            Value::Object(object)
        }
        Value::Array(results) => serde_json::json!({
            "results": results,
            "_aai_provider_next_url": next_url,
        }),
        other => serde_json::json!({
            "result": other,
            "_aai_provider_next_url": next_url,
        }),
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
        "pipedrive_personal_token" | "pipedrive-personal-token" => {
            let token = profile
                .api_token
                .as_deref()
                .or(profile.token.as_deref())
                .ok_or_else(|| {
                    AppError::auth(service, operation, "profile is missing api_token or token")
                })?;
            Ok(request.header("x-api-token", token))
        }
        "apollo_api_key" | "apollo-api-key" => {
            let token = profile
                .api_token
                .as_deref()
                .or(profile.token.as_deref())
                .ok_or_else(|| {
                    AppError::auth(service, operation, "profile is missing api_token or token")
                })?;
            Ok(request.header("x-api-key", token))
        }
        "openpanel_client_credentials" | "openpanel-client-credentials" => {
            let client_id = profile.client_id.as_deref().ok_or_else(|| {
                AppError::auth(service, operation, "profile is missing client_id")
            })?;
            let client_secret = profile
                .api_token
                .as_deref()
                .or(profile.token.as_deref())
                .ok_or_else(|| {
                    AppError::auth(
                        service,
                        operation,
                        "profile is missing api_token or token (the client secret)",
                    )
                })?;
            Ok(request
                .header("openpanel-client-id", client_id)
                .header("openpanel-client-secret", client_secret))
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

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Method;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    #[test]
    fn pipedrive_personal_token_uses_x_api_token_header() {
        let client = Client::new();
        let profile = Profile {
            auth_type: Some("pipedrive_personal_token".to_string()),
            api_token: Some("pd-token".to_string()),
            ..Profile::default()
        };
        let request = apply_auth(
            client.request(Method::GET, "https://api.pipedrive.com/api/v2/deals"),
            "pipedrive",
            "deals.list",
            &profile,
        )
        .unwrap()
        .build()
        .unwrap();

        assert_eq!(request.headers()["x-api-token"], "pd-token");
        assert!(!request.headers().contains_key("authorization"));
    }

    #[test]
    fn apollo_api_key_uses_x_api_key_header() {
        let client = Client::new();
        let profile = Profile {
            auth_type: Some("apollo_api_key".to_string()),
            api_token: Some("apollo-token".to_string()),
            ..Profile::default()
        };
        let request = apply_auth(
            client.request(
                Method::GET,
                "https://api.apollo.io/api/v1/users/api_profile",
            ),
            "apollo",
            "users.me",
            &profile,
        )
        .unwrap()
        .build()
        .unwrap();

        assert_eq!(request.headers()["x-api-key"], "apollo-token");
        assert!(!request.headers().contains_key("authorization"));
    }

    #[test]
    fn openpanel_client_credentials_uses_both_headers() {
        let client = Client::new();
        let profile = Profile {
            auth_type: Some("openpanel_client_credentials".to_string()),
            client_id: Some("018f0000-0000-0000-0000-000000000000".to_string()),
            api_token: Some("openpanel-secret".to_string()),
            ..Profile::default()
        };
        let request = apply_auth(
            client.request(Method::GET, "https://api.openpanel.dev/manage/projects"),
            "openpanel",
            "projects.list",
            &profile,
        )
        .unwrap()
        .build()
        .unwrap();

        assert_eq!(
            request.headers()["openpanel-client-id"],
            "018f0000-0000-0000-0000-000000000000"
        );
        assert_eq!(
            request.headers()["openpanel-client-secret"],
            "openpanel-secret"
        );
        assert!(!request.headers().contains_key("authorization"));
    }

    #[test]
    fn extracts_next_link_header() {
        assert_eq!(
            link_next_url(
                r#"<https://api.github.com/items?page=1>; rel="prev", <https://api.github.com/items?page=3>; rel="next""#
            )
            .as_deref(),
            Some("https://api.github.com/items?page=3")
        );
    }

    #[tokio::test]
    async fn no_redirect_requests_return_redirect_response_without_following() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request).unwrap();
            stream
                .write_all(
                    b"HTTP/1.1 302 Found\r\nLocation: http://127.0.0.1:9/redirected\r\nContent-Length: 0\r\n\r\n",
                )
                .unwrap();
        });
        let client = ApiClient::new().unwrap();
        let profile = Profile {
            auth_type: Some("none".to_string()),
            ..Profile::default()
        };

        let error = client
            .request_no_redirect(
                "test",
                "request",
                &profile,
                Method::GET,
                format!("http://{address}/start"),
                None,
            )
            .await
            .unwrap_err();
        server.join().unwrap();

        assert_eq!(error.code, "provider_api_error");
        assert_eq!(error.status, Some(302));
    }
}
