use anyhow::{Result, bail};
use async_openai::{Client, config::OpenAIConfig};

pub fn openai_compatible_client() -> Result<Client<OpenAIConfig>> {
    let provider = std::env::var("LOOPER_PROVIDER")
        .unwrap_or_else(|_| "openai".to_string())
        .trim()
        .to_ascii_lowercase();

    let api_base = std::env::var("LOOPER_BASE_URL")
        .ok()
        .or_else(|| std::env::var("OPENAI_BASE_URL").ok());

    let provider_api_key_var = format!("{}_API_KEY", provider_env_prefix(&provider));
    let api_key = std::env::var("LOOPER_API_KEY")
        .or_else(|_| std::env::var("OPENAI_API_KEY"))
        .or_else(|_| std::env::var(&provider_api_key_var))
        .map_err(|_| {
            anyhow::anyhow!(
                "Missing API key. Set LOOPER_API_KEY, OPENAI_API_KEY, or {}.",
                provider_api_key_var
            )
        })?;

    if api_key.trim().is_empty() {
        bail!("Provider API key is empty.");
    }

    let config = if let Some(api_base) = api_base {
        if api_base.trim().is_empty() {
            bail!("Provider base URL is empty. Set LOOPER_BASE_URL or OPENAI_BASE_URL.");
        }
        OpenAIConfig::new()
            .with_api_base(api_base)
            .with_api_key(api_key)
    } else if provider == "openai" {
        OpenAIConfig::new().with_api_key(api_key)
    } else {
        bail!(
            "LOOPER_PROVIDER='{}' requires LOOPER_BASE_URL (or OPENAI_BASE_URL).",
            provider
        );
    };

    Ok(Client::with_config(config))
}

fn provider_env_prefix(provider: &str) -> String {
    let prefix: String = provider
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect();

    if prefix.is_empty() {
        "PROVIDER".to_string()
    } else {
        prefix
    }
}
