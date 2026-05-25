/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::commands::app_data;
use crate::deploy::{
    deploy_game, purge_deployment, refresh_deploy_state, run_preflight, DeployGameRequest,
    DeployResult, DeployStateResponse, PurgeResult,
};
use crate::diagnostics::{DiagnosticReport, ModManifest};
use crate::errors::{AppError, UserFacingIssue};
use crate::hardlink::{check_same_partition, PartitionCheckResult};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{Emitter, Manager};

#[tauri::command]
pub fn check_partition(staging_dir: String, game_dir: String) -> Result<PartitionCheckResult, UserFacingIssue> {
    check_same_partition(&PathBuf::from(staging_dir), &PathBuf::from(game_dir))
        .map_err(|e| e.to_user_issue())
}

#[tauri::command]
pub fn run_preflight_checks(
    game_dir: String,
    profile_id: Option<String>,
    staging_dir: String,
    mods: Vec<ModManifest>,
    enabled_ids: Vec<String>,
    conflict_resolutions: HashMap<String, String>,
    deploy_path_override: Option<String>,
) -> DiagnosticReport {
    run_preflight(
        &game_dir,
        profile_id.as_deref(),
        &mods,
        &enabled_ids,
        &staging_dir,
        &conflict_resolutions,
        deploy_path_override.as_deref(),
    )
}

#[tauri::command]
pub fn deploy_game_mods(
    app: tauri::AppHandle,
    mut request: DeployGameRequest,
) -> Result<DeployResult, UserFacingIssue> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::user(e.to_string()).to_user_issue())?;

    let settings = crate::settings::load_settings(&app_data).unwrap_or_default();
    if settings.ignore_deploy_requirements {
        request.ignore_requirements = true;
    }
    if settings.auto_purge_before_deploy {
        let _ = purge_deployment(&app_data, &request.game_id);
    }

    let _ = app.emit(
        "deploy://started",
        &serde_json::json!({ "gameId": request.game_id }),
    );

    let result = deploy_game(
        &app_data,
        &request,
        settings.verify_after_deploy,
    )
    .map_err(|e| e.to_user_issue())?;

    let _ = app.emit(
        "deploy://completed",
        &serde_json::json!({
            "gameId": request.game_id,
            "summary": result.summary,
            "deployedFiles": result.deployed_files,
        }),
    );

    Ok(result)
}

#[tauri::command]
pub fn undeploy_mod(
    app: tauri::AppHandle,
    game_id: String,
    mod_id: String,
) -> Result<usize, UserFacingIssue> {
    let app_data = app_data(&app)?;
    crate::deploy::undeploy_mod(&app_data, &game_id, &mod_id).map_err(|e| e.to_user_issue())
}

#[tauri::command]
pub fn fix_bsa_timestamps(
    app: tauri::AppHandle,
    game_dir: String,
) -> Result<usize, UserFacingIssue> {
    let _ = app;
    crate::bethesda::fix_bsa_timestamps(std::path::Path::new(&game_dir))
        .map_err(|e| e.to_user_issue())
}

#[tauri::command]
pub fn get_deploy_state(
    app: tauri::AppHandle,
    game_id: String,
) -> Result<Option<DeployStateResponse>, UserFacingIssue> {
    let app_data = app_data(&app)?;
    refresh_deploy_state(&app_data, &game_id).map_err(|e| e.to_user_issue())
}

#[tauri::command]
pub fn purge_deployed_mods(
    app: tauri::AppHandle,
    game_id: String,
) -> Result<PurgeResult, UserFacingIssue> {
    let app_data = app_data(&app)?;
    purge_deployment(&app_data, &game_id).map_err(|e| e.to_user_issue())
}
