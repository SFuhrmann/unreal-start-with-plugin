# Unreal Plugin Launcher (Rust)

Small test project for coding assisted by Codex by OpenAI, it does what it should.

Standalone, cross-platform GUI + CLI launcher for Unreal projects with per-user plugin overrides. It keeps `.uproject` files untouched by using Unreal's `-EnablePlugins` and `-DisablePlugins` flags.

## Build
```powershell
cargo build --release
```

The binary will be at `target/release/unreal-plugin-launcher.exe` on Windows (or without `.exe` on macOS/Linux).

## Run
- GUI: run the binary without arguments.
- CLI: `--launch <path>`
```powershell
unreal-plugin-launcher --launch path\to\Project.uproject
```

## Associate .uproject
- GUI button: `Associate .uproject`
- CLI: `--associate`
```powershell
unreal-plugin-launcher --associate
```

## Config location
- Windows: `%APPDATA%\UnrealPluginLauncher\config.json`
- macOS: `~/Library/Application Support/UnrealPluginLauncher/config.json`
- Linux: `~/.config/unreal-plugin-launcher/config.json`

## Notes
- Engine detection scans common install locations and the Windows registry.
- If you use custom/source builds with non-standard paths, add them to `config.json` under the matching `EngineAssociation` value.

## For Collaborators
- `src/main.rs`: CLI entry point and launcher flow.
- `src/gui.rs`: GUI layout, plugin filtering, and async loading.
- `src/engine.rs`: engine discovery and EngineAssociation handling.
- `src/plugins.rs`: plugin scanning and per-engine override logic.
- `src/config.rs`: config persistence and storage location.
- `src/association.rs`: file association helpers.
- `src/lib.rs`: module wiring.

Development guidelines:
- Any EngineAssociation logic must remain stable because it is the config key.

## Ideas for the Future

- Login and access/install owned plugins
- General Profile for all UE versions
