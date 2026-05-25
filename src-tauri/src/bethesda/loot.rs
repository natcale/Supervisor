/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use super::PluginEntry;
use crate::errors::{AppError, AppResult};
use serde::Deserialize;
use std::path::Path;
use std::process::Command;

/// Basic LOOT-style sort: masters first, then alphabetical.
/// Full LOOT CLI integration replaces this when loot is installed.
pub fn sort_plugins(plugins: &mut [PluginEntry]) {
    plugins.sort_by(|a, b| {
        match (a.is_master, b.is_master) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });
}

#[cfg(test)]
pub fn plugin_order(plugins: &[PluginEntry]) -> Vec<String> {
    plugins.iter().map(|p| p.name.clone()).collect()
}

pub fn sort_plugins_with_loot(
    plugins: &mut [PluginEntry],
    profile_id: &str,
    loot_path: Option<&str>,
    game_path: &str,
) -> AppResult<bool> {
    let Some(loot) = loot_path.filter(|p| Path::new(p).is_file()) else {
        sort_plugins(plugins);
        return Ok(false);
    };

    let game_id = loot_game_id(profile_id);
    let output = Command::new(loot)
        .args([
            "--game",
            game_id,
            "--game-path",
            game_path,
            "--format",
            "json",
            "--resolve-conflicts",
        ])
        .output()
        .map_err(|e| AppError::user(format!("Could not run LOOT: {e}")))?;

    if !output.status.success() {
        sort_plugins(plugins);
        return Ok(false);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: LootJson = serde_json::from_str(&stdout).unwrap_or(LootJson {
        plugins: Vec::new(),
    });

    if parsed.plugins.is_empty() {
        sort_plugins(plugins);
        return Ok(false);
    }

    let rank: std::collections::HashMap<_, _> = parsed
        .plugins
        .iter()
        .enumerate()
        .map(|(i, p)| (p.name.to_lowercase(), i))
        .collect();

    plugins.sort_by_key(|p| rank.get(&p.name.to_lowercase()).copied().unwrap_or(usize::MAX));
    Ok(true)
}

fn loot_game_id(profile_id: &str) -> &'static str {
    match profile_id {
        "skyrimse" => "SkyrimSE",
        "skyrim" => "Skyrim",
        "fallout4" => "Fallout4",
        "newvegas" => "FalloutNV",
        "fallout3" => "Fallout3",
        "oblivion" => "Oblivion",
        "morrowind" => "Morrowind",
        "starfield" => "Starfield",
        _ => "SkyrimSE",
    }
}

#[derive(Debug, Deserialize)]
struct LootJson {
    #[serde(default)]
    plugins: Vec<LootPlugin>,
}

#[derive(Debug, Deserialize)]
struct LootPlugin {
    name: String,
}
