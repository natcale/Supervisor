/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use crate::bethesda::{self, PluginEntry};
use crate::commands::app_data;
use crate::diagnostics::ModManifest;
use crate::game_detection::DetectedGame;
use crate::games::resolve_profile;
use crate::settings::load_settings;

#[tauri::command]
pub fn get_plugin_list(
    game: DetectedGame,
    mods: Vec<ModManifest>,
    enabled_ids: Vec<String>,
) -> Vec<PluginEntry> {
    let profile = resolve_profile(&game);
    if !bethesda::profile_supports_plugins(profile) {
        return Vec::new();
    }
    let pairs: Vec<_> = mods
        .iter()
        .map(|m| (m.id.clone(), m.files.clone()))
        .collect();
    let mut plugins = bethesda::scan_plugins_from_mods(&pairs, &enabled_ids);
    if let Ok(states) = bethesda::plugin_states_from_txt(&game) {
        if !states.is_empty() {
            for plugin in &mut plugins {
                if let Some(enabled) = states.get(&plugin.name.to_lowercase()) {
                    plugin.enabled = *enabled;
                }
            }
        }
    }
    plugins
}

#[tauri::command]
pub fn sort_plugins_loot(
    app: tauri::AppHandle,
    game: DetectedGame,
    mods: Vec<ModManifest>,
    enabled_ids: Vec<String>,
) -> Vec<PluginEntry> {
    let mut plugins = get_plugin_list(game.clone(), mods, enabled_ids);
    let profile = resolve_profile(&game);
    let settings = app_data(&app)
        .ok()
        .and_then(|data| load_settings(&data).ok())
        .unwrap_or_default();
    let used_loot = bethesda::sort_plugins_with_loot(
        &mut plugins,
        &profile.id,
        settings.loot_path.as_deref(),
        &game.install_path,
    )
    .unwrap_or(false);
    if !used_loot {
        bethesda::sort_plugins(&mut plugins);
    }
    let enabled = bethesda::enabled_plugin_names(&plugins);
    let order = bethesda::plugin_order_names(&plugins);
    let _ = bethesda::write_plugins_txt(&game, &enabled, &order);
    plugins
}

#[tauri::command]
pub fn set_plugin_order(
    game: DetectedGame,
    mods: Vec<ModManifest>,
    enabled_ids: Vec<String>,
    order: Vec<String>,
) -> Vec<PluginEntry> {
    let mut plugins = get_plugin_list(game.clone(), mods, enabled_ids.clone());
    bethesda::apply_load_order(&mut plugins, &order);
    let enabled: Vec<String> = plugins
        .iter()
        .filter(|p| p.enabled)
        .map(|p| p.name.clone())
        .collect();
    let _ = bethesda::write_plugins_txt(&game, &enabled, &order);
    plugins
}

#[tauri::command]
pub fn toggle_plugin(
    game: DetectedGame,
    mods: Vec<ModManifest>,
    enabled_ids: Vec<String>,
    plugin_name: String,
    enabled: bool,
) -> Vec<PluginEntry> {
    let mut plugins = get_plugin_list(game.clone(), mods, enabled_ids);
    for p in &mut plugins {
        if p.name.eq_ignore_ascii_case(&plugin_name) {
            p.enabled = enabled;
        }
    }
    let order = bethesda::plugin_order_names(&plugins);
    let enabled_names = bethesda::enabled_plugin_names(&plugins);
    let _ = bethesda::write_plugins_txt(&game, &enabled_names, &order);
    plugins
}
