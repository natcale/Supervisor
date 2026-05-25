/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
#![allow(dead_code)]

use crate::deploy::DeployGameRequest;
use crate::errors::{AppError, AppResult};
use crate::game_detection::DetectedGame;
use crate::games::{GameProfile, resolve_profile};
use std::fs;
use std::path::{Path, PathBuf};

pub fn plugins_txt_path(game: &DetectedGame) -> Option<PathBuf> {
    let profile = resolve_profile(game);
    let folder = match profile.id.as_str() {
        "skyrimse" => "Skyrim Special Edition",
        "skyrim" => "Skyrim",
        "fallout4" => "Fallout4",
        "newvegas" => "FalloutNV",
        "fallout3" => "Fallout3",
        "oblivion" => "Oblivion",
        "morrowind" => "Morrowind",
        "starfield" => "Starfield",
        _ => return None,
    };

    #[cfg(windows)]
    {
        let local = std::env::var_os("LOCALAPPDATA")?;
        Some(PathBuf::from(local).join(folder).join("plugins.txt"))
    }
    #[cfg(not(windows))]
    {
        let _ = folder;
        None
    }
}

pub fn read_plugins_txt(game: &DetectedGame) -> AppResult<Vec<String>> {
    let Some(path) = plugins_txt_path(game) else {
        return Ok(Vec::new());
    };
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path).map_err(AppError::Io)?;
    Ok(parse_plugins_txt(&content))
}

pub fn write_plugins_txt(game: &DetectedGame, enabled: &[String], order: &[String]) -> AppResult<()> {
    let Some(path) = plugins_txt_path(game) else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(AppError::Io)?;
    }

    let enabled_set: std::collections::HashSet<_> = enabled.iter().map(|s| s.to_lowercase()).collect();
    let mut lines = Vec::new();

    for name in order {
        let lower = name.to_lowercase();
        if enabled_set.contains(&lower) {
            lines.push(format!("*{name}"));
        } else {
            lines.push(name.clone());
        }
    }

    fs::write(&path, lines.join("\n") + "\n").map_err(AppError::Io)
}

pub fn plugin_states_from_txt(game: &DetectedGame) -> AppResult<std::collections::HashMap<String, bool>> {
    let Some(path) = plugins_txt_path(game) else {
        return Ok(std::collections::HashMap::new());
    };
    if !path.is_file() {
        return Ok(std::collections::HashMap::new());
    }
    let content = fs::read_to_string(&path).map_err(AppError::Io)?;
    Ok(parse_plugins_txt_states(&content))
}

fn parse_plugins_txt_states(content: &str) -> std::collections::HashMap<String, bool> {
    let mut map = std::collections::HashMap::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let enabled = trimmed.starts_with('*');
        let name = trimmed.trim_start_matches('*').trim();
        if !name.is_empty() {
            map.insert(name.to_lowercase(), enabled);
        }
    }
    map
}

fn parse_plugins_txt(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            let enabled = trimmed.starts_with('*');
            let name = trimmed.trim_start_matches('*').trim();
            if name.is_empty() {
                None
            } else if enabled {
                Some(name.to_string())
            } else {
                None
            }
        })
        .collect()
}

pub fn enabled_plugin_names(plugins: &[super::PluginEntry]) -> Vec<String> {
    plugins
        .iter()
        .filter(|p| p.enabled)
        .map(|p| p.name.clone())
        .collect()
}

pub fn plugin_order_names(plugins: &[super::PluginEntry]) -> Vec<String> {
    plugins.iter().map(|p| p.name.clone()).collect()
}

pub fn sync_plugins_for_deploy(
    game_root: &Path,
    profile: &GameProfile,
    request: &DeployGameRequest,
    _staging: &Path,
) -> AppResult<()> {
    let _ = profile;
    let data = game_root.join("Data");
    if !data.is_dir() {
        return Ok(());
    }

    let game = DetectedGame {
        id: request.game_id.clone(),
        name: profile.name.clone(),
        platform: crate::game_detection::GamePlatform::Manual,
        install_path: game_root.to_string_lossy().into_owned(),
        executable: None,
        app_id: None,
        data_path: Some(data.to_string_lossy().into_owned()),
        nexus_domain: None,
        profile_id: Some(profile.id.clone()),
    };

    let Some(path) = plugins_txt_path(&game) else {
        return Ok(());
    };
    if !path.is_file() {
        return Ok(());
    }

    let content = fs::read_to_string(&path).map_err(AppError::Io)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut changed = false;

    lines.retain(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return true;
        }
        let name = trimmed.trim_start_matches('*').trim();
        let plugin_path = data.join(name);
        if plugin_path.is_file() {
            true
        } else {
            changed = true;
            false
        }
    });

    if changed {
        fs::write(&path, lines.join("\n") + "\n").map_err(AppError::Io)?;
    }
    Ok(())
}
