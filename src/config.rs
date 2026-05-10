use std::{collections::HashMap, env, fs, path::PathBuf};

use serde::Deserialize;

use crate::error::AppError;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub default_profile: Option<String>,
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Profile {
    pub provider: Option<String>,
    pub transport: Option<String>,
    pub auth_type: Option<String>,
    pub base_url: Option<String>,
    pub site_url: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub token: Option<String>,
    pub token_env: Option<String>,
    pub api_token: Option<String>,
    pub api_token_env: Option<String>,
    pub password: Option<String>,
    pub password_env: Option<String>,
    pub workspace: Option<String>,
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub org: Option<String>,
    pub user_id: Option<String>,
    pub account_id: Option<String>,
    pub calendar_id: Option<String>,
    pub from_address: Option<String>,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub imap_host: Option<String>,
    pub imap_port: Option<u16>,
    pub mail_folder: Option<String>,
    pub sent_folder: Option<String>,
    pub caldav_url: Option<String>,
}

pub struct Context {
    pub profile: Profile,
}

impl Context {
    pub fn load(config_arg: Option<&str>, profile_arg: Option<&str>) -> Result<Self, AppError> {
        let config_path = config_path(config_arg)?;
        let text = if config_path.exists() {
            fs::read_to_string(&config_path).map_err(|err| {
                AppError::config(format!("failed to read {}: {err}", config_path.display()))
            })?
        } else {
            String::new()
        };
        let config: Config = if text.trim().is_empty() {
            Config::default()
        } else {
            toml::from_str(&text)
                .map_err(|err| AppError::config(format!("invalid config TOML: {err}")))?
        };

        let profile_name = profile_arg
            .map(str::to_string)
            .or_else(|| env::var("AAI_PROFILE").ok())
            .or(config.default_profile)
            .unwrap_or_else(|| "default".to_string());

        let mut profile = config
            .profiles
            .get(&profile_name)
            .cloned()
            .unwrap_or_default();
        apply_env_overrides(&mut profile, &profile_name);

        Ok(Self { profile })
    }
}

fn config_path(config_arg: Option<&str>) -> Result<PathBuf, AppError> {
    if let Some(path) = config_arg {
        return Ok(PathBuf::from(path));
    }
    if let Ok(path) = env::var("AAI_CONFIG") {
        return Ok(PathBuf::from(path));
    }
    let config_dir =
        dirs::config_dir().ok_or_else(|| AppError::config("could not locate config directory"))?;
    Ok(config_dir.join("aai-cli").join("config.toml"))
}

fn apply_env_overrides(profile: &mut Profile, profile_name: &str) {
    if profile.token.is_none() {
        profile.token = profile.token_env.as_deref().and_then(env_value);
    }
    if profile.api_token.is_none() {
        profile.api_token = profile.api_token_env.as_deref().and_then(env_value);
    }
    if profile.password.is_none() {
        profile.password = profile.password_env.as_deref().and_then(env_value);
    }

    let normalized = profile_name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect::<String>();

    if profile.token.is_none() {
        profile.token =
            env_value(&format!("AAI_{}_TOKEN", normalized)).or_else(|| env_value("AAI_TOKEN"));
    }
    if profile.api_token.is_none() {
        profile.api_token = env_value(&format!("AAI_{}_API_TOKEN", normalized))
            .or_else(|| env_value("AAI_API_TOKEN"));
    }
    if profile.password.is_none() {
        profile.password = env_value(&format!("AAI_{}_PASSWORD", normalized))
            .or_else(|| env_value("AAI_PASSWORD"));
    }
}

fn env_value(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_uses_default_profile() {
        let config: Config = toml::from_str(
            r#"
default_profile = "work"

[profiles.work]
provider = "github"
token = "abc"
"#,
        )
        .unwrap();
        assert_eq!(config.default_profile.as_deref(), Some("work"));
        assert_eq!(config.profiles["work"].provider.as_deref(), Some("github"));
    }
}
