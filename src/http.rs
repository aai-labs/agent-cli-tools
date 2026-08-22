use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::{engine::general_purpose, Engine as _};
use reqwest::{Client, Method, RequestBuilder};
use serde_json::Value;

use crate::gateway_catalog;
use crate::gateway_protocol::{
    ExecuteMetadata, GatewayErrorBody, EXECUTE_PATH, PROTOCOL_VERSION, RESPONSE_SOURCE_HEADER,
};
use crate::{config::Profile, error::AppError};

pub(crate) fn multipart_boundary() -> String {
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
        if profile.credential_source.as_deref() == Some("gateway") {
            let body = body
                .map(|value| serde_json::to_vec(&value))
                .transpose()
                .map_err(|err| AppError::internal(service, operation, err.to_string()))?;
            let response = self
                .gateway_execute(GatewayExecuteRequest {
                    service,
                    operation,
                    profile,
                    method,
                    url,
                    body,
                    content_type: Some("application/json"),
                    accept: Some("application/json"),
                    headers: Vec::new(),
                })
                .await?;
            return parse_json_response(service, operation, response);
        }
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
        if profile.credential_source.as_deref() == Some("gateway") {
            let body = body
                .map(|value| serde_json::to_vec(&value))
                .transpose()
                .map_err(|err| AppError::internal(service, operation, err.to_string()))?;
            let response = self
                .gateway_execute(GatewayExecuteRequest {
                    service,
                    operation,
                    profile,
                    method,
                    url,
                    body,
                    content_type: Some("application/json"),
                    accept: Some("application/json"),
                    headers: Vec::new(),
                })
                .await?;
            return parse_json_response(service, operation, response);
        }
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
        if profile.credential_source.as_deref() == Some("gateway") {
            let response = self
                .gateway_execute(GatewayExecuteRequest {
                    service,
                    operation,
                    profile,
                    method: Method::GET,
                    url,
                    body: None,
                    content_type: None,
                    accept: Some(accept),
                    headers: Vec::new(),
                })
                .await?;
            return response_bytes(service, operation, response);
        }
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
        if profile.credential_source.as_deref() == Some("gateway") {
            let BytesRequest {
                method,
                url,
                content_type,
                headers,
                body,
            } = request;
            let response = self
                .gateway_execute(GatewayExecuteRequest {
                    service,
                    operation,
                    profile,
                    method,
                    url,
                    body: Some(body),
                    content_type: Some(&content_type),
                    accept: Some("application/json"),
                    headers,
                })
                .await?;
            let body = response_bytes(service, operation, response)?;
            return Ok(RawResponse {
                body: parse_body_value(&body),
                location: None,
            });
        }
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

        let content_type = format!("multipart/form-data; boundary={boundary}");
        if profile.credential_source.as_deref() == Some("gateway") {
            let response = self
                .gateway_execute(GatewayExecuteRequest {
                    service,
                    operation,
                    profile,
                    method: Method::POST,
                    url,
                    body: Some(body),
                    content_type: Some(&content_type),
                    accept: Some("application/json"),
                    headers: vec![("X-Atlassian-Token".to_string(), "no-check".to_string())],
                })
                .await?;
            return parse_json_response(service, operation, response);
        }
        let token = crate::oauth::resolve_token(profile, &self.client, service, operation).await?;
        let effective = crate::config::Profile {
            token: Some(token),
            ..profile.clone()
        };
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

    async fn gateway_execute(
        &self,
        request: GatewayExecuteRequest<'_>,
    ) -> Result<GatewayResponse, AppError> {
        let GatewayExecuteRequest {
            service,
            operation,
            profile,
            method,
            url,
            body,
            content_type,
            accept,
            headers,
        } = request;
        if !gateway_catalog::allows(service, operation, &method) {
            return Err(AppError::invalid_input(
                service,
                operation,
                "gateway mode only supports valid typed operations",
            ));
        }
        let gateway_url = profile.gateway_url.as_deref().ok_or_else(|| {
            AppError::auth(service, operation, "gateway profile is missing gateway_url")
        })?;
        let profile_id = profile.gateway_profile_id.as_deref().ok_or_else(|| {
            AppError::auth(
                service,
                operation,
                "gateway profile is missing gateway_profile_id",
            )
        })?;
        let token = profile.gateway_token.as_deref().ok_or_else(|| {
            AppError::auth(
                service,
                operation,
                "gateway profile is missing gateway token",
            )
        })?;
        let metadata = ExecuteMetadata {
            protocol_version: PROTOCOL_VERSION,
            profile_id: profile_id.to_string(),
            service: service.to_string(),
            operation: operation.to_string(),
            method: method.to_string(),
            url,
            content_type: content_type.map(str::to_string),
            accept: accept.map(str::to_string),
            headers: headers.clone(),
        };
        let metadata_json = serde_json::to_string(&metadata).map_err(|err| {
            AppError::internal(
                service,
                operation,
                format!("failed to encode gateway request: {err}"),
            )
        })?;
        let metadata_part = reqwest::multipart::Part::text(metadata_json)
            .mime_str("application/json")
            .map_err(|err| AppError::internal(service, operation, err.to_string()))?;
        let mut form = reqwest::multipart::Form::new().part("metadata", metadata_part);
        if let Some(body) = body {
            form = form.part("body", reqwest::multipart::Part::bytes(body));
        }
        let endpoint = format!("{}{}", gateway_url.trim_end_matches('/'), EXECUTE_PATH);
        let response = self
            .client
            .post(endpoint)
            .bearer_auth(token)
            .multipart(form)
            .send()
            .await
            .map_err(|err| {
                AppError::internal(service, operation, format!("gateway request failed: {err}"))
            })?;
        let status = response.status();
        let source = response
            .headers()
            .get(RESPONSE_SOURCE_HEADER)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("gateway")
            .to_string();
        let body = response.bytes().await.map_err(|err| {
            AppError::internal(
                service,
                operation,
                format!("failed to read gateway response: {err}"),
            )
        })?;
        if !status.is_success() {
            if source == "provider" {
                let details = serde_json::from_slice(&body).ok().or_else(|| {
                    Some(Value::String(
                        String::from_utf8_lossy(&body).chars().take(4096).collect(),
                    ))
                });
                return Err(AppError::api(
                    service,
                    operation,
                    status,
                    format!("provider returned HTTP {}", status.as_u16()),
                    details,
                ));
            }
            let details = serde_json::from_slice::<GatewayErrorBody>(&body)
                .ok()
                .and_then(|error| serde_json::to_value(error).ok());
            return Err(AppError::gateway(
                service,
                operation,
                status,
                format!("gateway rejected request (HTTP {})", status.as_u16()),
                details,
            ));
        }
        Ok(GatewayResponse {
            status,
            body: body.to_vec(),
        })
    }
}

struct GatewayExecuteRequest<'a> {
    service: &'static str,
    operation: &'static str,
    profile: &'a Profile,
    method: Method,
    url: String,
    body: Option<Vec<u8>>,
    content_type: Option<&'a str>,
    accept: Option<&'a str>,
    headers: Vec<(String, String)>,
}

struct GatewayResponse {
    status: reqwest::StatusCode,
    body: Vec<u8>,
}

fn parse_body_value(body: &[u8]) -> Value {
    if body.is_empty() {
        Value::Object(serde_json::Map::new())
    } else {
        serde_json::from_slice(body)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(body).into_owned()))
    }
}

fn parse_json_response(
    service: &'static str,
    operation: &'static str,
    response: GatewayResponse,
) -> Result<Value, AppError> {
    if !response.status.is_success() {
        return Err(AppError::api(
            service,
            operation,
            response.status,
            format!("provider returned HTTP {}", response.status.as_u16()),
            Some(parse_body_value(&response.body)),
        ));
    }
    Ok(parse_body_value(&response.body))
}

fn response_bytes(
    service: &'static str,
    operation: &'static str,
    response: GatewayResponse,
) -> Result<Vec<u8>, AppError> {
    if response.status.is_success() {
        Ok(response.body)
    } else {
        Err(AppError::api(
            service,
            operation,
            response.status,
            format!("provider returned HTTP {}", response.status.as_u16()),
            Some(parse_body_value(&response.body)),
        ))
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

pub(crate) fn apply_auth(
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
        "hubspot_service_key"
        | "hubspot-service-key"
        | "hubspot_legacy_private_app"
        | "hubspot-legacy-private-app" => {
            let token = profile
                .token
                .as_deref()
                .or(profile.api_token.as_deref())
                .ok_or_else(|| AppError::auth(service, operation, "profile is missing token"))?;
            Ok(request.bearer_auth(token))
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
        sync::{Arc, Mutex},
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

    async fn provider_once(
        body: &'static [u8],
        content_type: &'static str,
    ) -> (String, Arc<Mutex<Option<String>>>) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let authorization = Arc::new(Mutex::new(None));
        let captured = Arc::clone(&authorization);
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut bytes = vec![0_u8; 8192];
            let count = stream.read(&mut bytes).await.unwrap();
            let request = String::from_utf8_lossy(&bytes[..count]);
            let value = request
                .lines()
                .find_map(|line| {
                    line.strip_prefix("authorization: ")
                        .or_else(|| line.strip_prefix("Authorization: "))
                })
                .map(str::to_string);
            *captured.lock().unwrap() = value;
            let head = format!("HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
            stream.write_all(head.as_bytes()).await.unwrap();
            stream.write_all(body).await.unwrap();
        });
        (format!("http://{address}"), authorization)
    }

    async fn gateway_profile(
        provider_url: &str,
        operation: &str,
    ) -> (String, String, tempfile::TempDir) {
        use crate::gateway_server::{router, GatewayApp, GatewayStore};
        let temp = tempfile::tempdir().unwrap();
        let store = GatewayStore::open(
            temp.path().join("state.enc.json"),
            temp.path().join("state.key"),
        )
        .unwrap();
        let client = Client::builder().build().unwrap();
        let app = GatewayApp {
            store: Arc::new(tokio::sync::RwLock::new(store)),
            admin_token: "admin".to_string(),
            client: client.clone(),
        };
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, router(app)).await.unwrap();
        });
        let gateway_url = format!("http://{address}");
        let profile = Profile {
            provider: Some("github".into()),
            auth_type: Some("bearer_token".into()),
            base_url: Some(provider_url.into()),
            gateway_profile_id: Some("fake".into()),
            token: Some("provider-secret".into()),
            ..Profile::default()
        };
        client
            .post(format!("{gateway_url}/api/v1/admin/profiles"))
            .bearer_auth("admin")
            .json(&serde_json::json!({"profile": profile}))
            .send()
            .await
            .unwrap();
        let token_response: Value = client.post(format!("{gateway_url}/api/v1/admin/tokens")).bearer_auth("admin").json(&serde_json::json!({"grants": {"fake": {"allow_operations": [operation], "allow_methods": ["GET", "POST"]}}})).send().await.unwrap().json().await.unwrap();
        (
            gateway_url,
            token_response["token"].as_str().unwrap().to_string(),
            temp,
        )
    }

    #[tokio::test]
    async fn direct_json_request_keeps_provider_auth_local() {
        let (provider, authorization) = provider_once(br#"{"ok":true}"#, "application/json").await;
        let client = ApiClient::new().unwrap();
        let profile = Profile {
            base_url: Some(provider.clone()),
            auth_type: Some("bearer_token".into()),
            token: Some("provider-secret".into()),
            ..Profile::default()
        };
        let value = client
            .request(
                "github",
                "items.list",
                &profile,
                Method::GET,
                format!("{provider}/items"),
                None,
            )
            .await
            .unwrap();
        assert_eq!(value["ok"], true);
        assert_eq!(
            authorization.lock().unwrap().as_deref(),
            Some("Bearer provider-secret")
        );
    }

    #[tokio::test]
    async fn gateway_json_request_injects_provider_auth_without_returning_it() {
        let (provider, authorization) = provider_once(
            br#"{"ok":true,"echo":"provider-secret"}"#,
            "application/json",
        )
        .await;
        let (gateway, proxy_token, _temp) = gateway_profile(&provider, "items.list").await;
        let client = ApiClient::new().unwrap();
        let profile = Profile {
            credential_source: Some("gateway".into()),
            gateway_url: Some(gateway),
            gateway_profile_id: Some("fake".into()),
            gateway_token: Some(proxy_token),
            base_url: Some(provider.clone()),
            auth_type: Some("bearer_token".into()),
            ..Profile::default()
        };
        let value = client
            .request(
                "github",
                "items.list",
                &profile,
                Method::GET,
                format!("{provider}/items"),
                None,
            )
            .await
            .unwrap();
        assert_eq!(value["ok"], true);
        assert_eq!(value["echo"], "[REDACTED]");
        assert_eq!(
            authorization.lock().unwrap().as_deref(),
            Some("Bearer provider-secret")
        );
    }

    #[tokio::test]
    async fn direct_download_returns_provider_bytes() {
        let (provider, authorization) =
            provider_once(b"provider-secret", "application/octet-stream").await;
        let client = ApiClient::new().unwrap();
        let profile = Profile {
            base_url: Some(provider.clone()),
            auth_type: Some("bearer_token".into()),
            token: Some("provider-secret".into()),
            ..Profile::default()
        };
        let bytes = client
            .download(
                "github",
                "items.download",
                &profile,
                format!("{provider}/items/1"),
            )
            .await
            .unwrap();
        assert_eq!(bytes, b"provider-secret");
        assert_eq!(
            authorization.lock().unwrap().as_deref(),
            Some("Bearer provider-secret")
        );
    }

    #[tokio::test]
    async fn gateway_download_returns_redacted_bytes() {
        let (provider, authorization) = provider_once(b"provider-secret", "text/plain").await;
        let (gateway, proxy_token, _temp) = gateway_profile(&provider, "items.download").await;
        let client = ApiClient::new().unwrap();
        let profile = Profile {
            credential_source: Some("gateway".into()),
            gateway_url: Some(gateway),
            gateway_profile_id: Some("fake".into()),
            gateway_token: Some(proxy_token),
            base_url: Some(provider.clone()),
            auth_type: Some("bearer_token".into()),
            ..Profile::default()
        };
        let bytes = client
            .download(
                "github",
                "items.download",
                &profile,
                format!("{provider}/items/1"),
            )
            .await
            .unwrap();
        assert_eq!(bytes, b"[REDACTED]");
        assert_eq!(
            authorization.lock().unwrap().as_deref(),
            Some("Bearer provider-secret")
        );
    }

    #[tokio::test]
    async fn direct_bytes_request_uses_provider_auth() {
        let (provider, authorization) =
            provider_once(br#"{"uploaded":true}"#, "application/json").await;
        let client = ApiClient::new().unwrap();
        let profile = Profile {
            base_url: Some(provider.clone()),
            auth_type: Some("bearer_token".into()),
            token: Some("provider-secret".into()),
            ..Profile::default()
        };
        let response = client
            .request_bytes(
                "github",
                "items.upload",
                &profile,
                BytesRequest {
                    method: Method::POST,
                    url: format!("{provider}/items"),
                    content_type: "application/octet-stream".into(),
                    headers: vec![],
                    body: b"data".to_vec(),
                },
            )
            .await
            .unwrap();
        assert_eq!(response.body["uploaded"], true);
        assert_eq!(
            authorization.lock().unwrap().as_deref(),
            Some("Bearer provider-secret")
        );
    }

    #[tokio::test]
    async fn gateway_bytes_request_uses_proxy_and_redacts_response() {
        let (provider, authorization) = provider_once(
            br#"{"uploaded":true,"key":"provider-secret"}"#,
            "application/json",
        )
        .await;
        let (gateway, proxy_token, _temp) = gateway_profile(&provider, "items.upload").await;
        let client = ApiClient::new().unwrap();
        let profile = Profile {
            credential_source: Some("gateway".into()),
            gateway_url: Some(gateway),
            gateway_profile_id: Some("fake".into()),
            gateway_token: Some(proxy_token),
            base_url: Some(provider.clone()),
            auth_type: Some("bearer_token".into()),
            ..Profile::default()
        };
        let response = client
            .request_bytes(
                "github",
                "items.upload",
                &profile,
                BytesRequest {
                    method: Method::POST,
                    url: format!("{provider}/items"),
                    content_type: "application/octet-stream".into(),
                    headers: vec![],
                    body: b"data".to_vec(),
                },
            )
            .await
            .unwrap();
        assert_eq!(response.body["uploaded"], true);
        assert_eq!(response.body["key"], "[REDACTED]");
        assert_eq!(
            authorization.lock().unwrap().as_deref(),
            Some("Bearer provider-secret")
        );
    }
}
