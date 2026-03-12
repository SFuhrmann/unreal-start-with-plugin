use clap::Parser;
use std::path::PathBuf;

use unreal_plugin_launcher::association::associate_uproject;
use unreal_plugin_launcher::config::load_config;
use unreal_plugin_launcher::engine::{detect_engines, editor_executable, find_engine_path, read_engine_association};
use unreal_plugin_launcher::gui::LauncherApp;
use unreal_plugin_launcher::plugins::plugin_overrides;

// CLI front-end for launching and file association.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Launch a uproject using configured plugin overrides
    #[arg(long)]
    launch: Option<PathBuf>,

    /// Associate .uproject files with this launcher (current user)
    #[arg(long)]
    associate: bool,
}

fn launch_project(uproject: &PathBuf) -> Result<(), String> {
    if !uproject.exists() {
        return Err(format!("Uproject not found: {uproject:?}"));
    }

    let engine_version = read_engine_association(uproject)
        .ok_or_else(|| "Could not read EngineAssociation from uproject.".to_string())?;

    let config = load_config();
    let detected = detect_engines();
    let engine_path = find_engine_path(&engine_version, &config, &detected)
        .ok_or_else(|| format!("Could not find Unreal Engine for {engine_version}."))?;

    let editor_exe = editor_executable(&engine_path);
    if !editor_exe.exists() {
        return Err(format!("UnrealEditor not found at {editor_exe:?}."));
    }

    let (enable, disable) = plugin_overrides(&config, &engine_version);

    let mut cmd = std::process::Command::new(editor_exe);
    cmd.arg(uproject);
    if !enable.is_empty() {
        cmd.arg(format!("-EnablePlugins={}", enable.join(",")));
    }
    if !disable.is_empty() {
        cmd.arg(format!("-DisablePlugins={}", disable.join(",")));
    }
    cmd.current_dir(
        uproject
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".")),
    );

    cmd.spawn().map_err(|e| e.to_string())?;
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    if cli.associate {
        match associate_uproject() {
            Ok(msg) => {
                println!("{msg}");
                return;
            }
            Err(msg) => {
                eprintln!("{msg}");
                std::process::exit(1);
            }
        }
    }

    if let Some(uproject) = cli.launch {
        if let Err(err) = launch_project(&uproject) {
            eprintln!("{err}");
            std::process::exit(1);
        }
        return;
    }

    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Unreal Plugin Launcher",
        options,
        Box::new(|_cc| Box::new(LauncherApp::new())),
    );
}
