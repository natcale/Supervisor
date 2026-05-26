/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use crate::commands::{launch_detected_game, staging_dir_for};
use crate::errors::UserFacingIssue;
use crate::game_detection::{manual, scan_all_games, DetectedGame, GameScanResult};
use crate::games::{profile_summary, resolve_profile};
use crate::settings::load_settings;
use std::path::Path;

fn resolve_include_all(app_data: &Path, include_all: Option<bool>) -> bool {
    include_all.unwrap_or_else(|| {
        load_settings(app_data)
            .unwrap_or_default()
            .show_unmoddable_games
    })
}

#[tauri::command]
pub fn scan_games(
    app: tauri::AppHandle,
    include_all: Option<bool>,
) -> Result<GameScanResult, UserFacingIssue> {
    let app_data = crate::commands::app_data(&app)?;
    let include_all = resolve_include_all(&app_data, include_all);
    scan_all_games(&app_data, include_all).map_err(|e| e.to_user_issue())
}

#[tauri::command]
pub fn add_manual_game(
    app: tauri::AppHandle,
    install_path: String,
    name: Option<String>,
) -> Result<DetectedGame, UserFacingIssue> {
    let app_data = crate::commands::app_data(&app)?;
    let mut game =
        manual::add_manual_game(&app_data, install_path, name).map_err(|e| e.to_user_issue())?;
    crate::games::attach_profile(&mut game);
    Ok(game)
}

#[tauri::command]
pub fn get_deploy_targets(game: DetectedGame) -> Vec<crate::games::DeployTargetSummary> {
    crate::games::list_deploy_targets(&game)
}

#[tauri::command]
pub fn get_game_profile(game: DetectedGame) -> crate::games::GameProfileSummary {
    let profile = resolve_profile(&game);
    profile_summary(profile)
}

#[tauri::command]
pub fn list_supported_profiles() -> Vec<crate::games::GameProfileSummary> {
    crate::games::all_profile_summaries()
}

#[tauri::command]
pub fn remove_manual_game(app: tauri::AppHandle, game_id: String) -> Result<(), UserFacingIssue> {
    let app_data = crate::commands::app_data(&app)?;
    manual::remove_manual_game(&app_data, &game_id).map_err(|e| e.to_user_issue())
}

#[tauri::command]
pub fn update_manual_game_nexus_domain(
    app: tauri::AppHandle,
    game_id: String,
    nexus_domain: Option<String>,
) -> Result<DetectedGame, UserFacingIssue> {
    let app_data = crate::commands::app_data(&app)?;
    let mut game = manual::update_manual_game_nexus_domain(&app_data, &game_id, nexus_domain)
        .map_err(|e| e.to_user_issue())?;
    crate::games::attach_profile(&mut game);
    Ok(game)
}

#[tauri::command]
pub fn get_staging_dir(app: tauri::AppHandle, game_id: String) -> Result<String, UserFacingIssue> {
    Ok(staging_dir_for(&app, &game_id)?
        .to_string_lossy()
        .into_owned())
}

#[tauri::command]
pub fn launch_game(app: tauri::AppHandle, game: DetectedGame) -> Result<(), UserFacingIssue> {
    launch_detected_game(&app, &game).map_err(|e| e.to_user_issue())
}
