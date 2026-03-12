pub fn associate_uproject() -> Result<String, String> {
    if cfg!(windows) {
        return associate_windows();
    }
    if cfg!(target_os = "macos") {
        return Err(
            "Association is not automated on macOS. Use a packaged .app or set it manually.".to_string(),
        );
    }
    associate_linux()
}

#[cfg(windows)]
fn associate_windows() -> Result<String, String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let command = format!("\"{}\" --launch \"%1\"", exe.display());

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes = hkcu
        .create_subkey("Software\\Classes")
        .map_err(|e| e.to_string())?
        .0;

    let (ext_key, _) = classes
        .create_subkey(".uproject")
        .map_err(|e| e.to_string())?;
    ext_key
        .set_value("", &"UnrealProjectLauncher")
        .map_err(|e| e.to_string())?;

    let (cmd_key, _) = classes
        .create_subkey("UnrealProjectLauncher\\shell\\open\\command")
        .map_err(|e| e.to_string())?;
    cmd_key
        .set_value("", &command)
        .map_err(|e| e.to_string())?;

    Ok("Associated .uproject files for current user.".to_string())
}

#[cfg(not(windows))]
fn associate_windows() -> Result<String, String> {
    Err("Windows association is not supported on this platform.".to_string())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn associate_linux() -> Result<String, String> {
    use std::path::PathBuf;
    use std::process::Command;

    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let home = std::env::var_os("HOME").map(PathBuf::from).ok_or("HOME not set")?;
    let desktop_dir = home.join(".local").join("share").join("applications");
    std::fs::create_dir_all(&desktop_dir).map_err(|e| e.to_string())?;

    let desktop_file = desktop_dir.join("unreal-project-launcher.desktop");
    let desktop_content = format!(
        "[Desktop Entry]\nType=Application\nName=Unreal Project Launcher\nExec={} --launch %f\nMimeType=application/x-unreal-project;\nNoDisplay=true\n",
        exe.display()
    );
    std::fs::write(&desktop_file, desktop_content).map_err(|e| e.to_string())?;

    let status = Command::new("xdg-mime")
        .arg("default")
        .arg(desktop_file.file_name().unwrap())
        .arg("application/x-unreal-project")
        .status();

    match status {
        Ok(s) if s.success() => Ok("Associated .uproject files for current user.".to_string()),
        Ok(s) => Err(format!("xdg-mime failed with status {s}")),
        Err(e) => Err(format!("xdg-mime failed: {e}")),
    }
}

#[cfg(not(all(unix, not(target_os = "macos"))))]
fn associate_linux() -> Result<String, String> {
    Err("Linux association is not supported on this platform.".to_string())
}
