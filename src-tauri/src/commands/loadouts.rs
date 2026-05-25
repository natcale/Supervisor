/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::commands::app_data;
use crate::errors::UserFacingIssue;
use crate::loadouts::{Loadout, LoadoutSummary};

#[tauri::command]
pub fn list_loadouts(
    app: tauri::AppHandle,
    game_id: String,
) -> Result<(Vec<LoadoutSummary>, String), UserFacingIssue> {
    let data = app_data(&app)?;
    crate::loadouts::list_loadouts(&data, &game_id).map_err(|e| e.to_user_issue())
}

#[tauri::command]
pub fn switch_loadout(
    app: tauri::AppHandle,
    game_id: String,
    loadout_id: String,
) -> Result<Loadout, UserFacingIssue> {
    let data = app_data(&app)?;
    crate::loadouts::switch_loadout(&data, &game_id, &loadout_id).map_err(|e| e.to_user_issue())
}

#[tauri::command]
pub fn create_loadout(
    app: tauri::AppHandle,
    game_id: String,
    name: String,
) -> Result<Loadout, UserFacingIssue> {
    let data = app_data(&app)?;
    crate::loadouts::create_loadout(&data, &game_id, &name).map_err(|e| e.to_user_issue())
}

#[tauri::command]
pub fn update_loadout(
    app: tauri::AppHandle,
    game_id: String,
    loadout: Loadout,
) -> Result<Loadout, UserFacingIssue> {
    let data = app_data(&app)?;
    crate::loadouts::update_loadout(&data, &game_id, loadout).map_err(|e| e.to_user_issue())
}

#[tauri::command]
pub fn delete_loadout(
    app: tauri::AppHandle,
    game_id: String,
    loadout_id: String,
) -> Result<(), UserFacingIssue> {
    let data = app_data(&app)?;
    crate::loadouts::delete_loadout(&data, &game_id, &loadout_id).map_err(|e| e.to_user_issue())
}
