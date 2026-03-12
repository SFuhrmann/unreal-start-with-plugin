use crate::association::associate_uproject;
use crate::config::{config_path, load_config, save_config, Config};
use crate::engine::{detect_engines, EngineInfo};
use crate::plugins::{get_plugin_state, list_plugins, set_plugin_state, PluginInfo};
use eframe::{egui, App};
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver};

// Egui-based GUI that edits per-user plugin overrides.
pub struct LauncherApp {
    config: Config,
    engines: Vec<EngineInfo>,
    selected: Option<usize>,
    plugins: Vec<PluginInfo>,
    plugin_cache: HashMap<String, Vec<PluginInfo>>,
    status: String,
    plugin_filter: String,
    loading_plugins: bool,
    plugin_rx: Option<Receiver<Vec<PluginInfo>>>,
    loading_version: Option<String>,
}

impl LauncherApp {
    pub fn new() -> Self {
        Self {
            config: load_config(),
            engines: detect_engines(),
            selected: None,
            plugins: Vec::new(),
            plugin_cache: HashMap::new(),
            status: String::new(),
            plugin_filter: String::new(),
            loading_plugins: false,
            plugin_rx: None,
            loading_version: None,
        }
    }

    fn refresh_versions(&mut self) {
        self.engines = detect_engines();
        self.selected = None;
        self.plugins.clear();
        self.plugin_cache.clear();
        self.loading_plugins = false;
        self.plugin_rx = None;
        self.loading_version = None;
    }

    fn load_plugins_for_selected(&mut self) {
        self.plugins.clear();
        if let Some(idx) = self.selected {
            if let Some(engine) = self.engines.get(idx) {
                let engine_id = engine.id.clone();
                if let Some(cached) = self.plugin_cache.get(&engine_id) {
                    self.plugins = cached.clone();
                    self.loading_plugins = false;
                    self.plugin_rx = None;
                    self.loading_version = None;
                    return;
                }

                let engine_path = engine.path.clone();
                let (tx, rx) = mpsc::channel();
                self.loading_plugins = true;
                self.plugin_rx = Some(rx);
                self.loading_version = Some(engine_id);

                // Scan plugins on a background thread to keep UI responsive.
                std::thread::spawn(move || {
                    let plugins = list_plugins(&engine_path);
                    let _ = tx.send(plugins);
                });
            }
        }
    }

    fn selected_engine_id(&self) -> Option<String> {
        self.selected
            .and_then(|idx| self.engines.get(idx).map(|e| e.id.clone()))
    }

    fn selected_engine_label(&self) -> Option<String> {
        self.selected.and_then(|idx| {
            self.engines.get(idx).map(|e| {
                if e.id == e.display_version {
                    e.display_version.clone()
                } else {
                    format!("{} ({})", e.display_version, e.id)
                }
            })
        })
    }
}

impl App for LauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.loading_plugins {
            if let Some(rx) = &self.plugin_rx {
                if let Ok(plugins) = rx.try_recv() {
                    self.plugins = plugins;
                    if let Some(version) = &self.loading_version {
                        self.plugin_cache.insert(version.clone(), self.plugins.clone());
                    }
                    self.loading_plugins = false;
                    self.plugin_rx = None;
                    self.loading_version = None;
                }
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Associate .uproject").clicked() {
                    match associate_uproject() {
                        Ok(msg) => self.status = msg,
                        Err(msg) => self.status = msg,
                    }
                }

                if ui.button("Refresh Versions").clicked() {
                    self.refresh_versions();
                }

                ui.label(format!("Config: {}", config_path().display()));
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |columns| {
                let left = &mut columns[0];
                left.heading("Unreal Engine Versions");

                let mut new_selection = self.selected;
                egui::ScrollArea::vertical()
                    .id_source("engine_versions_scroll")
                    .show(left, |ui| {
                        for (idx, engine) in self.engines.iter().enumerate() {
                            let label = if engine.id == engine.display_version {
                                format!("{} | {}", engine.display_version, engine.path.display())
                            } else {
                                format!(
                                    "{} ({}) | {}",
                                    engine.display_version,
                                    engine.id,
                                    engine.path.display()
                                )
                            };
                            let selected = new_selection == Some(idx);
                            if ui.selectable_label(selected, label).clicked() {
                                new_selection = Some(idx);
                            }
                        }
                    });

                if new_selection != self.selected {
                    self.selected = new_selection;
                    self.load_plugins_for_selected();
                }

                let right = &mut columns[1];
                if let Some(engine_id) = self.selected_engine_id() {
                    let label = self
                        .selected_engine_label()
                        .unwrap_or_else(|| engine_id.clone());
                    right.heading(format!("Plugins for {label}"));

                    if right.button("Save Changes").clicked() {
                        match save_config(&self.config) {
                            Ok(_) => self.status = "Configuration saved.".to_string(),
                            Err(e) => self.status = format!("Save failed: {e}"),
                        }
                    }

                    right.label("State: Default = no override, Enable = force on, Disable = force off");
                    right.add(
                        egui::TextEdit::singleline(&mut self.plugin_filter)
                            .hint_text("Filter plugins..."),
                    );

                    if self.loading_plugins {
                        right.horizontal(|ui| {
                            ui.add(egui::Spinner::new());
                            ui.label("Loading plugins...");
                        });
                    }

                    let filter = self.plugin_filter.trim().to_lowercase();
                    let mut engine_plugins: Vec<&PluginInfo> = Vec::new();
                    let mut marketplace_plugins: Vec<&PluginInfo> = Vec::new();
                    for plugin in &self.plugins {
                        if !filter.is_empty() && !plugin.name.to_lowercase().contains(&filter) {
                            continue;
                        }
                        if plugin.is_marketplace {
                            marketplace_plugins.push(plugin);
                        } else {
                            engine_plugins.push(plugin);
                        }
                    }

                    egui::ScrollArea::vertical()
                        .id_source("plugins_scroll")
                        .show(right, |ui| {
                            egui::CollapsingHeader::new("Engine Plugins")
                                .default_open(true)
                                .show(ui, |ui| {
                                    for plugin in engine_plugins {
                                        ui.horizontal(|ui| {
                                            ui.label(&plugin.name);

                                            let current =
                                                get_plugin_state(&self.config, &engine_id, &plugin.name);
                                            let mut selected = current.clone();
                                            egui::ComboBox::from_id_source(format!(
                                                "engine:{}:{}",
                                                engine_id, plugin.name
                                            ))
                                            .selected_text(match selected.as_str() {
                                                "enable" => "enable",
                                                "disable" => "disable",
                                                _ => "default",
                                            })
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(
                                                    &mut selected,
                                                    "default".to_string(),
                                                    "default",
                                                );
                                                ui.selectable_value(
                                                    &mut selected,
                                                    "enable".to_string(),
                                                    "enable",
                                                );
                                                ui.selectable_value(
                                                    &mut selected,
                                                    "disable".to_string(),
                                                    "disable",
                                                );
                                            });

                                            if selected != current {
                                                set_plugin_state(
                                                    &mut self.config,
                                                    &engine_id,
                                                    &plugin.name,
                                                    &selected,
                                                );
                                            }
                                        });
                                    }
                                });

                            egui::CollapsingHeader::new("Marketplace Plugins")
                                .default_open(true)
                                .show(ui, |ui| {
                                    for plugin in marketplace_plugins {
                                        ui.horizontal(|ui| {
                                            ui.label(&plugin.name);

                                            let current =
                                                get_plugin_state(&self.config, &engine_id, &plugin.name);
                                            let mut selected = current.clone();
                                            egui::ComboBox::from_id_source(format!(
                                                "marketplace:{}:{}",
                                                engine_id, plugin.name
                                            ))
                                            .selected_text(match selected.as_str() {
                                                "enable" => "enable",
                                                "disable" => "disable",
                                                _ => "default",
                                            })
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(
                                                    &mut selected,
                                                    "default".to_string(),
                                                    "default",
                                                );
                                                ui.selectable_value(
                                                    &mut selected,
                                                    "enable".to_string(),
                                                    "enable",
                                                );
                                                ui.selectable_value(
                                                    &mut selected,
                                                    "disable".to_string(),
                                                    "disable",
                                                );
                                            });

                                            if selected != current {
                                                set_plugin_state(
                                                    &mut self.config,
                                                    &engine_id,
                                                    &plugin.name,
                                                    &selected,
                                                );
                                            }
                                        });
                                    }
                                });
                        });
                } else {
                    right.heading("Select a version");
                }
            });
        });

        if !self.status.is_empty() {
            egui::TopBottomPanel::bottom("status_panel").show(ctx, |ui| {
                ui.label(&self.status);
            });
        }
    }
}
