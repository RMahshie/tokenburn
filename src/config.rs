use std::path::PathBuf;

use color_eyre::eyre::{eyre, Result};

pub const APP_NAME: &str = "tokenburn";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn home_dir() -> Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| eyre!("could not determine home directory"))
}

pub fn claude_glob() -> Result<String> {
    Ok(home_dir()?
        .join(".claude/projects/**/*.jsonl")
        .to_string_lossy()
        .into_owned())
}

pub fn codex_glob() -> Result<String> {
    Ok(home_dir()?
        .join(".codex/sessions/*/*/*/*.jsonl")
        .to_string_lossy()
        .into_owned())
}
