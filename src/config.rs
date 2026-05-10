use std::path::PathBuf;

use color_eyre::eyre::{eyre, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};

pub const APP_NAME: &str = "tokenburn";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CLAUDE_RETENTION_DAYS: u64 = 3650;

pub fn home_dir() -> Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| eyre!("could not determine home directory"))
}

pub fn tokenburn_config_path() -> Result<PathBuf> {
    let base = dirs::config_dir().unwrap_or(home_dir()?.join(".config"));
    Ok(base.join(APP_NAME).join("config.json"))
}

pub fn claude_glob() -> Result<String> {
    Ok(home_dir()?
        .join(".claude/projects/**/*.jsonl")
        .to_string_lossy()
        .into_owned())
}

pub fn claude_settings_path() -> Result<PathBuf> {
    Ok(home_dir()?.join(".claude/settings.json"))
}

pub fn claude_stats_cache_path() -> Result<PathBuf> {
    Ok(home_dir()?.join(".claude/stats-cache.json"))
}

pub fn codex_glob() -> Result<String> {
    Ok(home_dir()?
        .join(".codex/sessions/*/*/*/*.jsonl")
        .to_string_lossy()
        .into_owned())
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenburnConfig {
    #[serde(default)]
    pub suppress_claude_retention_prompt: bool,
}

pub fn tokenburn_config() -> Result<TokenburnConfig> {
    let path = tokenburn_config_path()?;
    match std::fs::read_to_string(&path) {
        Ok(contents) => Ok(serde_json::from_str(&contents)?),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(TokenburnConfig::default()),
        Err(err) => Err(err.into()),
    }
}

pub fn set_suppress_claude_retention_prompt(value: bool) -> Result<TokenburnConfig> {
    let path = tokenburn_config_path()?;
    let mut config = tokenburn_config()?;
    config.suppress_claude_retention_prompt = value;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut output = serde_json::to_string_pretty(&config)?;
    output.push('\n');
    std::fs::write(path, output)?;

    Ok(config)
}

#[derive(Debug, Clone)]
pub struct ClaudeRetentionStatus {
    pub path: PathBuf,
    pub cleanup_period_days: Option<u64>,
    pub needs_update: bool,
}

pub fn claude_retention_status() -> Result<ClaudeRetentionStatus> {
    let path = claude_settings_path()?;
    let cleanup_period_days = match std::fs::read_to_string(&path) {
        Ok(contents) => parse_cleanup_period_days(&contents)?,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
        Err(err) => return Err(err.into()),
    };

    Ok(ClaudeRetentionStatus {
        path,
        cleanup_period_days,
        needs_update: cleanup_period_days.unwrap_or(30) < CLAUDE_RETENTION_DAYS,
    })
}

pub fn set_claude_retention(days: u64) -> Result<ClaudeRetentionStatus> {
    let path = claude_settings_path()?;
    let mut settings = match std::fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str::<Value>(&contents)?,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Value::Object(Map::new()),
        Err(err) => return Err(err.into()),
    };

    let object = settings
        .as_object_mut()
        .ok_or_else(|| eyre!("{} must contain a JSON object", path.display()))?;
    object.insert(
        "cleanupPeriodDays".to_string(),
        Value::Number(Number::from(days)),
    );

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut output = serde_json::to_string_pretty(&settings)?;
    output.push('\n');
    std::fs::write(&path, output)?;

    claude_retention_status()
}

fn parse_cleanup_period_days(contents: &str) -> Result<Option<u64>> {
    let value = serde_json::from_str::<Value>(contents)?;
    Ok(value
        .get("cleanupPeriodDays")
        .and_then(|days| days.as_u64()))
}
