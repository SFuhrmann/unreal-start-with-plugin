use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

// Engine installation metadata used by the GUI and CLI.
#[derive(Debug, Clone)]
pub struct EngineInfo {
    pub id: String,
    pub display_version: String,
    pub path: PathBuf,
}

pub fn read_build_version(engine_path: &Path) -> Option<String> {
    let build_file = engine_path.join("Engine").join("Build").join("Build.version");
    let text = fs::read_to_string(build_file).ok()?;
    let json: Value = serde_json::from_str(&text).ok()?;
    let major = json.get("MajorVersion")?.as_u64()?;
    let minor = json.get("MinorVersion")?.as_u64()?;
    let patch = json.get("PatchVersion").and_then(|v| v.as_u64());
    let version = if let Some(patch) = patch {
        format!("{major}.{minor}.{patch}")
    } else {
        format!("{major}.{minor}")
    };
    Some(version)
}

fn normalize_version_from_folder(name: &str) -> Option<String> {
    if let Some(rest) = name.strip_prefix("UE_") {
        return Some(rest.to_string());
    }
    if let Some(rest) = name.strip_prefix("UnrealEngine-") {
        return Some(rest.to_string());
    }
    None
}

fn add_engine(engines: &mut Vec<EngineInfo>, id: String, display_version: String, path: PathBuf) {
    if path.exists() {
        engines.push(EngineInfo {
            id,
            display_version,
            path,
        });
    }
}

pub fn detect_engines() -> Vec<EngineInfo> {
    let mut engines = Vec::new();

    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let paths = [
            r"SOFTWARE\EpicGames\Unreal Engine\Builds",
            r"SOFTWARE\WOW6432Node\EpicGames\Unreal Engine\Builds",
        ];

        for key_path in paths {
            if let Ok(key) = hklm.open_subkey(key_path) {
                for item in key.enum_values().flatten() {
                    let (name, value) = item;
                    let value_str: String = value.to_string();
                    if value_str.is_empty() {
                        continue;
                    }
                    let path = PathBuf::from(value_str);
                    let display_version = read_build_version(&path).unwrap_or_else(|| name.clone());
                    add_engine(&mut engines, name, display_version, path);
                }
            }
        }

        for base in ["C:/Program Files/Epic Games", "C:/Program Files (x86)/Epic Games"] {
            let base = PathBuf::from(base);
            if !base.exists() {
                continue;
            }
            if let Ok(entries) = fs::read_dir(base) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    let name = entry.file_name().to_string_lossy().to_string();
                    if !name.starts_with("UE_") {
                        continue;
                    }
                    let id = normalize_version_from_folder(&name).unwrap_or(name);
                    let display_version = read_build_version(&path).unwrap_or_else(|| id.clone());
                    add_engine(&mut engines, id, display_version, path);
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        for base in ["/Users/Shared/Epic Games", "/Applications"] {
            let base = PathBuf::from(base);
            if !base.exists() {
                continue;
            }
            if let Ok(entries) = fs::read_dir(base) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    let name = entry.file_name().to_string_lossy().to_string();
                    if !name.starts_with("UE_") {
                        continue;
                    }
                    let id = normalize_version_from_folder(&name).unwrap_or(name);
                    let display_version = read_build_version(&path).unwrap_or_else(|| id.clone());
                    add_engine(&mut engines, id, display_version, path);
                }
            }
        }
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/"));
        for base in [home.join("EpicGames"), PathBuf::from("/opt")] {
            if !base.exists() {
                continue;
            }
            if let Ok(entries) = fs::read_dir(base) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    let name = entry.file_name().to_string_lossy().to_string();
                    if !name.starts_with("UE_") {
                        continue;
                    }
                    let id = normalize_version_from_folder(&name).unwrap_or(name);
                    let display_version = read_build_version(&path).unwrap_or_else(|| id.clone());
                    add_engine(&mut engines, id, display_version, path);
                }
            }
        }
    }

    engines.sort_by(|a, b| {
        a.display_version
            .to_lowercase()
            .cmp(&b.display_version.to_lowercase())
    });
    engines
}

pub fn read_engine_association(uproject_path: &Path) -> Option<String> {
    let text = fs::read_to_string(uproject_path).ok()?;
    let json: Value = serde_json::from_str(&text).ok()?;
    json.get("EngineAssociation")?.as_str().map(|s| s.to_string())
}

pub fn find_engine_path(
    version: &str,
    config: &crate::config::Config,
    detected: &[EngineInfo],
) -> Option<PathBuf> {
    if let Some(engine_cfg) = config.engines.get(version) {
        if let Some(path) = &engine_cfg.path {
            let p = PathBuf::from(path);
            if p.exists() {
                return Some(p);
            }
        }
    }

    if let Some(found) = detected.iter().find(|e| e.id == version) {
        return Some(found.path.clone());
    }

    if let Some(found) = detected.iter().find(|e| e.display_version == version) {
        return Some(found.path.clone());
    }

    None
}

pub fn editor_executable(engine_path: &Path) -> PathBuf {
    if cfg!(windows) {
        return engine_path
            .join("Engine")
            .join("Binaries")
            .join("Win64")
            .join("UnrealEditor.exe");
    }
    if cfg!(target_os = "macos") {
        return engine_path
            .join("Engine")
            .join("Binaries")
            .join("Mac")
            .join("UnrealEditor.app")
            .join("Contents")
            .join("MacOS")
            .join("UnrealEditor");
    }
    engine_path
        .join("Engine")
        .join("Binaries")
        .join("Linux")
        .join("UnrealEditor")
}
