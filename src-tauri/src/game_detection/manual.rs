/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use crate::errors::{AppError, AppResult};
use crate::game_detection::types::{DetectedGame, GamePlatform};
use crate::games::resolve_profile_for_detected_name;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManualGameRecord {
    id: String,
    name: String,
    install_path: String,
    executable: Option<String>,
    data_path: Option<String>,
    nexus_domain: Option<String>,
    profile_id: Option<String>,
}

fn manual_games_path(app_data: &Path) -> PathBuf {
    app_data.join("manual_games.json")
}

pub fn load_manual_games(app_data: &Path) -> AppResult<Vec<DetectedGame>> {
    let path = manual_games_path(app_data);
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&path).map_err(AppError::Io)?;
    let records: Vec<ManualGameRecord> =
        serde_json::from_str(&raw).map_err(|e| AppError::user(format!("Bad manual games: {e}")))?;
    Ok(records
        .into_iter()
        .filter(|r| PathBuf::from(&r.install_path).is_dir())
        .map(|r| DetectedGame {
            id: r.id,
            name: r.name,
            platform: GamePlatform::Manual,
            install_path: r.install_path,
            executable: r.executable,
            app_id: None,
            data_path: r.data_path,
            nexus_domain: r.nexus_domain,
            profile_id: r.profile_id,
        })
        .collect())
}

fn load_manual_records(app_data: &Path) -> AppResult<Vec<ManualGameRecord>> {
    let path = manual_games_path(app_data);
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&path).map_err(AppError::Io)?;
    serde_json::from_str(&raw).map_err(|e| AppError::user(format!("Bad manual games: {e}")))
}

pub fn add_manual_game(
    app_data: &Path,
    install_path: String,
    name: Option<String>,
) -> AppResult<DetectedGame> {
    let path = PathBuf::from(&install_path);
    if !path.is_dir() {
        return Err(AppError::user("Install folder does not exist."));
    }

    let display_name = name.unwrap_or_else(|| {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Manual Game")
            .to_string()
    });

    let id = format!("manual-{}", uuid::Uuid::new_v4().simple().to_string());

    let executable = find_executable(&path);
    let data_path = infer_data_path(&path);
    let (profile_id, nexus_domain) = resolve_profile_for_detected_name(&display_name)
        .map(|(id, domain)| (Some(id.to_string()), Some(domain.to_string())))
        .unwrap_or((None, None));

    let game = DetectedGame {
        id: id.clone(),
        name: display_name,
        platform: GamePlatform::Manual,
        install_path,
        executable,
        app_id: None,
        data_path,
        nexus_domain,
        profile_id,
    };

    let mut games = load_manual_records(app_data)?;
    games.retain(|g| g.install_path != game.install_path);
    games.push(ManualGameRecord {
        id: game.id.clone(),
        name: game.name.clone(),
        install_path: game.install_path.clone(),
        executable: game.executable.clone(),
        data_path: game.data_path.clone(),
        nexus_domain: game.nexus_domain.clone(),
        profile_id: game.profile_id.clone(),
    });

    fs::create_dir_all(app_data).map_err(AppError::Io)?;
    let raw = serde_json::to_string_pretty(&games).map_err(|e| AppError::user(e.to_string()))?;
    fs::write(manual_games_path(app_data), raw).map_err(AppError::Io)?;

    Ok(game)
}

pub fn remove_manual_game(app_data: &Path, game_id: &str) -> AppResult<()> {
    let mut games = load_manual_records(app_data)?;
    let before = games.len();
    games.retain(|g| g.id != game_id);
    if games.len() == before {
        return Err(AppError::user("Game not found or is not a local entry."));
    }
    let raw = serde_json::to_string_pretty(&games).map_err(|e| AppError::user(e.to_string()))?;
    fs::write(manual_games_path(app_data), raw).map_err(AppError::Io)?;
    Ok(())
}

pub fn update_manual_game_nexus_domain(
    app_data: &Path,
    game_id: &str,
    nexus_domain: Option<String>,
) -> AppResult<DetectedGame> {
    let mut games = load_manual_records(app_data)?;
    let record = games
        .iter_mut()
        .find(|g| g.id == game_id)
        .ok_or_else(|| AppError::user("Game not found or is not a local entry."))?;

    let domain = nexus_domain
        .map(|d| d.trim().to_lowercase())
        .filter(|d| !d.is_empty());
    record.nexus_domain = domain.clone();
    record.profile_id = domain
        .as_deref()
        .and_then(crate::games::profile_id_for_domain)
        .map(str::to_string);

    let game = DetectedGame {
        id: record.id.clone(),
        name: record.name.clone(),
        platform: GamePlatform::Manual,
        install_path: record.install_path.clone(),
        executable: record.executable.clone(),
        app_id: None,
        data_path: record.data_path.clone(),
        nexus_domain: record.nexus_domain.clone(),
        profile_id: record.profile_id.clone(),
    };

    let raw = serde_json::to_string_pretty(&games).map_err(|e| AppError::user(e.to_string()))?;
    fs::write(manual_games_path(app_data), raw).map_err(AppError::Io)?;
    Ok(game)
}

fn find_executable(install_path: &Path) -> Option<String> {
    if let Ok(entries) = fs::read_dir(install_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if path.extension().and_then(|e| e.to_str()) == Some("exe") {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if !name.starts_with("unins") {
                        return Some(path.to_string_lossy().into_owned());
                    }
                }
            }
        }
    }
    None
}

fn infer_data_path(install_path: &Path) -> Option<String> {
    for sub in ["Data", "Mods", "BepInEx/plugins", "mod"] {
        let p = install_path.join(sub);
        if p.is_dir() {
            return Some(p.to_string_lossy().into_owned());
        }
    }
    Some(install_path.join("Data").to_string_lossy().into_owned())
}
