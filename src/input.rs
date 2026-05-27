use std::{fs, io::Read};

use serde_json::{json, Map, Value};

use crate::error::AppError;

pub fn read_json_arg(
    service: &'static str,
    operation: &'static str,
    json_arg: Option<&str>,
) -> Result<Value, AppError> {
    let Some(json_arg) = json_arg else {
        return Ok(Value::Object(Map::new()));
    };

    let text = if json_arg == "-" {
        let mut text = String::new();
        std::io::stdin().read_to_string(&mut text)?;
        text
    } else if json_arg.trim_start().starts_with('{') || json_arg.trim_start().starts_with('[') {
        json_arg.to_string()
    } else {
        fs::read_to_string(json_arg).map_err(|err| {
            AppError::invalid_input(
                service,
                operation,
                format!("failed to read JSON file {json_arg}: {err}"),
            )
        })?
    };

    serde_json::from_str(&text).map_err(|err| {
        AppError::invalid_input(service, operation, format!("invalid JSON payload: {err}"))
    })
}

pub fn set_string(value: &mut Value, key: &str, field: &Option<String>) {
    if let Some(field) = field {
        ensure_object(value).insert(key.to_string(), Value::String(field.clone()));
    }
}

pub fn set_u64(value: &mut Value, key: &str, field: Option<u64>) {
    if let Some(field) = field {
        ensure_object(value).insert(key.to_string(), json!(field));
    }
}

pub fn ensure_object(value: &mut Value) -> &mut Map<String, Value> {
    if !value.is_object() {
        *value = Value::Object(Map::new());
    }
    value
        .as_object_mut()
        .expect("value was just made an object")
}

pub fn minimal_adf(text: &str) -> Value {
    json!({
        "type": "doc",
        "version": 1,
        "content": [
            {
                "type": "paragraph",
                "content": [
                    {
                        "type": "text",
                        "text": text
                    }
                ]
            }
        ]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_inline_json() {
        let value = read_json_arg("test", "parse", Some(r#"{"a":1}"#)).unwrap();
        assert_eq!(value["a"], 1);
    }

    #[test]
    fn creates_minimal_adf_document() {
        let value = minimal_adf("hello");
        assert_eq!(value["type"], "doc");
        assert_eq!(value["content"][0]["content"][0]["text"], "hello");
    }
}
