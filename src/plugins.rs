use serde_json::Value;
use std::path::Path;
use walkdir::WalkDir;

// Simple plugin representation with Marketplace grouping.
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub is_marketplace: bool,
}

pub fn list_plugins(engine_path: &Path) -> Vec<PluginInfo> {
    let plugins_dir = engine_path.join("Engine").join("Plugins");
    if !plugins_dir.exists() {
        return Vec::new();
    }

    let mut plugins: Vec<PluginInfo> = Vec::new();
    for entry in WalkDir::new(plugins_dir).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|v| v.to_str()) != Some("uplugin") {
            continue;
        }

        let is_marketplace = entry
            .path()
            .components()
            .any(|c| c.as_os_str().to_string_lossy().eq_ignore_ascii_case("Marketplace"));

        let name = std::fs::read_to_string(entry.path())
            .ok()
            .and_then(|text| serde_json::from_str::<Value>(&text).ok())
            .and_then(|json| json.get("Name").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .unwrap_or_else(|| {
                entry
                    .path()
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            });

        plugins.push(PluginInfo { name, is_marketplace });
    }

    plugins.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    plugins.dedup_by(|a, b| a.name.eq_ignore_ascii_case(&b.name));
    plugins
}

pub fn get_plugin_state(config: &crate::config::Config, version: &str, plugin: &str) -> String {
    config
        .engines
        .get(version)
        .and_then(|e| e.plugins.get(plugin))
        .cloned()
        .unwrap_or_else(|| "default".to_string())
}

pub fn set_plugin_state(
    config: &mut crate::config::Config,
    version: &str,
    plugin: &str,
    state: &str,
) {
    let engine = config.engines.entry(version.to_string()).or_default();
    if state == "default" {
        engine.plugins.remove(plugin);
    } else {
        engine.plugins.insert(plugin.to_string(), state.to_string());
    }
}

pub fn plugin_overrides(config: &crate::config::Config, version: &str) -> (Vec<String>, Vec<String>) {
    let mut enable = Vec::new();
    let mut disable = Vec::new();

    if let Some(engine) = config.engines.get(version) {
        for (name, state) in &engine.plugins {
            match state.as_str() {
                "enable" => enable.push(name.clone()),
                "disable" => disable.push(name.clone()),
                _ => {}
            }
        }
    }

    enable.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    disable.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    (enable, disable)
}
