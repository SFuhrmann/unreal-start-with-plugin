use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// Per-user persistent configuration keyed by EngineAssociation.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Config {
    pub engines: HashMap<String, EngineConfig>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct EngineConfig {
    pub path: Option<String>,
    pub plugins: HashMap<String, String>,
}

pub fn config_path() -> PathBuf {
    if let Some(proj) = directories::ProjectDirs::from("com", "local", "UnrealPluginLauncher") {
        return proj.config_dir().join("config.json");
    }

    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".config")
        .join("unreal-plugin-launcher")
        .join("config.json")
}

pub fn load_config() -> Config {
    let path = config_path();
    let data = fs::read_to_string(path);
    match data {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

pub fn save_config(cfg: &Config) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| e.to_string())
}
