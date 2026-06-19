use std::collections::HashMap;

use base64::{engine::general_purpose, Engine as _};

pub(super) fn decode_base64url(s: &str) -> String {
    general_purpose::URL_SAFE_NO_PAD
        .decode(s)
        .ok()
        .and_then(|b| String::from_utf8(b).ok())
        .unwrap_or_else(|| String::from_utf8_lossy(&general_purpose::URL_SAFE_NO_PAD.decode(s).unwrap_or_default()).into_owned())
}

pub(super) fn decode_base64_standard(s: &str) -> String {
    let cleaned: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    general_purpose::STANDARD
        .decode(&cleaned)
        .ok()
        .and_then(|b| String::from_utf8(b).ok())
        .unwrap_or_else(|| String::from_utf8_lossy(&general_purpose::STANDARD.decode(&cleaned).unwrap_or_default()).into_owned())
}

pub(super) fn decode_qp(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'=' && i + 1 < bytes.len() {
            let next = bytes[i + 1];
            if next == b'\r' || next == b'\n' {
                // soft line break — skip '=' and the newline(s)
                i += 1;
                if i < bytes.len() && bytes[i] == b'\r' {
                    i += 1;
                }
                if i < bytes.len() && bytes[i] == b'\n' {
                    i += 1;
                }
                continue;
            } else if i + 2 < bytes.len() {
                let hex = &s[i + 1..i + 3];
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    out.push(byte);
                    i += 3;
                    continue;
                }
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub(super) fn strip_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    let decoded = decode_html_entities(&out);
    collapse_whitespace(&decoded)
}

fn decode_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .replace("&#39;", "'")
        .replace("&quot;", "\"")
        .replace("&#34;", "\"")
        .replace("&#38;", "&")
        .replace("&apos;", "'")
}

fn collapse_whitespace(s: &str) -> String {
    let mut out = String::new();
    let mut prev_blank = false;
    for line in s.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !prev_blank && !out.is_empty() {
                out.push('\n');
            }
            prev_blank = true;
        } else {
            out.push_str(trimmed);
            out.push('\n');
            prev_blank = false;
        }
    }
    out.trim_end().to_string()
}

/// Parse RFC 2822 headers into a lowercase-keyed map.
/// Handles folded header lines (continuation lines starting with whitespace).
pub(super) fn parse_rfc822_headers(text: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut current_key: Option<String> = None;
    let mut current_val = String::new();

    for line in text.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            // folded continuation
            current_val.push(' ');
            current_val.push_str(line.trim());
        } else {
            if let Some(key) = current_key.take() {
                map.insert(key, current_val.trim().to_string());
                current_val = String::new();
            }
            if let Some(colon) = line.find(':') {
                current_key = Some(line[..colon].trim().to_lowercase());
                current_val = line[colon + 1..].trim().to_string();
            }
        }
    }
    if let Some(key) = current_key {
        map.insert(key, current_val.trim().to_string());
    }
    map
}

/// Split an RFC 2822 message into (headers_text, body_text).
pub(super) fn split_rfc822(raw: &str) -> (&str, &str) {
    if let Some(pos) = raw.find("\r\n\r\n") {
        return (&raw[..pos], &raw[pos + 4..]);
    }
    if let Some(pos) = raw.find("\n\n") {
        return (&raw[..pos], &raw[pos + 2..]);
    }
    (raw, "")
}

/// Extract the boundary value from a Content-Type header value like
/// `multipart/alternative; boundary="abc123"`.
pub(super) fn extract_boundary(content_type: &str) -> Option<String> {
    for part in content_type.split(';') {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix("boundary=") {
            return Some(rest.trim_matches('"').to_string());
        }
    }
    None
}

/// Walk a Gmail API payload node recursively to find the best readable body.
/// Prefers text/plain over text/html.
pub(super) fn find_body_in_gmail_payload(part: &serde_json::Value) -> Option<(String, &'static str)> {
    use serde_json::Value;

    let mime = part.get("mimeType").and_then(|v| v.as_str()).unwrap_or("");
    let data = part
        .get("body")
        .and_then(|b| b.get("data"))
        .and_then(|d| d.as_str())
        .filter(|s| !s.is_empty());

    if let Some(data) = data {
        if mime == "text/plain" {
            return Some((decode_base64url(data), "text"));
        }
        if mime == "text/html" {
            return Some((strip_html(&decode_base64url(data)), "html"));
        }
    }

    let mut html_fallback: Option<String> = None;
    if let Some(Value::Array(parts)) = part.get("parts") {
        for child in parts {
            if let Some((body, bt)) = find_body_in_gmail_payload(child) {
                if bt == "text" {
                    return Some((body, "text"));
                }
                if html_fallback.is_none() {
                    html_fallback = Some(body);
                }
            }
        }
    }
    html_fallback.map(|b| (b, "html"))
}

/// Recursively find the best readable body from an RFC 2822 / MIME message.
/// Returns plain text body (text/plain preferred over text/html).
pub(super) fn extract_readable_body(raw: &str) -> (String, &'static str) {
    let (header_text, body_text) = split_rfc822(raw);
    let headers = parse_rfc822_headers(header_text);

    let content_type = headers.get("content-type").map(|s| s.as_str()).unwrap_or("text/plain");
    let encoding = headers.get("content-transfer-encoding").map(|s| s.to_lowercase());
    let encoding = encoding.as_deref().unwrap_or("7bit");

    if content_type.starts_with("multipart/") {
        if let Some(boundary) = extract_boundary(content_type) {
            return extract_multipart_body(body_text, &boundary);
        }
    }

    let decoded = apply_transfer_encoding(body_text, encoding);

    if content_type.starts_with("text/html") {
        (strip_html(&decoded), "html")
    } else {
        (decoded.trim_end().to_string(), "text")
    }
}

fn extract_multipart_body(body: &str, boundary: &str) -> (String, &'static str) {
    let delimiter = format!("--{boundary}");
    let mut html_fallback: Option<String> = None;

    for part in body.split(delimiter.as_str()) {
        let part = part.trim_start_matches(['\r', '\n']);
        if part.starts_with("--") || part.is_empty() {
            continue;
        }
        let (header_text, part_body) = split_rfc822(part);
        let headers = parse_rfc822_headers(header_text);
        let ct = headers.get("content-type").map(|s| s.as_str()).unwrap_or("");
        let enc = headers.get("content-transfer-encoding").map(|s| s.to_lowercase());
        let enc = enc.as_deref().unwrap_or("7bit");

        if ct.starts_with("text/plain") {
            let decoded = apply_transfer_encoding(part_body, enc);
            return (decoded.trim_end().to_string(), "text");
        }
        if ct.starts_with("text/html") && html_fallback.is_none() {
            let decoded = apply_transfer_encoding(part_body, enc);
            html_fallback = Some(strip_html(&decoded));
        }
        if ct.starts_with("multipart/") {
            if let Some(inner_boundary) = extract_boundary(ct) {
                let (body, bt) = extract_multipart_body(part_body, &inner_boundary);
                if bt == "text" {
                    return (body, "text");
                }
                if html_fallback.is_none() {
                    html_fallback = Some(body);
                }
            }
        }
    }

    (html_fallback.unwrap_or_default(), "html")
}

fn apply_transfer_encoding(body: &str, encoding: &str) -> String {
    match encoding {
        "base64" => decode_base64_standard(body),
        "quoted-printable" => decode_qp(body),
        _ => body.to_string(),
    }
}
