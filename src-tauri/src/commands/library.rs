/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::commands::{
    app_data, open_path_in_shell, persist_ingested, staging_dir_for, GameStateResponse,
};
use crate::errors::{AppError, UserFacingIssue};
use crate::ingest::{self, IngestedMod};
use crate::install::FomodConfig;
use crate::library::GameLibrary;

#[tauri::command]
pub fn get_game_state(
    app: tauri::AppHandle,
    game_id: String,
) -> Result<GameStateResponse, UserFacingIssue> {
    let data = app_data(&app)?;
    let staging = staging_dir_for(&app, &game_id)?;
    let library = crate::library::get_library(&data, &game_id).map_err(|e| e.to_user_issue())?;
    let loadout =
        crate::loadouts::get_active_loadout(&data, &game_id).map_err(|e| e.to_user_issue())?;
    Ok(GameStateResponse {
        library,
        loadout,
        staging_dir: staging.to_string_lossy().into_owned(),
    })
}

#[tauri::command]
pub fn get_library(app: tauri::AppHandle, game_id: String) -> Result<GameLibrary, UserFacingIssue> {
    let data = app_data(&app)?;
    crate::library::get_library(&data, &game_id).map_err(|e| e.to_user_issue())
}

#[tauri::command]
pub fn remove_library_mod(
    app: tauri::AppHandle,
    game_id: String,
    mod_id: String,
) -> Result<GameLibrary, UserFacingIssue> {
    let data = app_data(&app)?;
    let staging = staging_dir_for(&app, &game_id)?;
    crate::library::remove_mod(&data, &staging, &game_id, &mod_id).map_err(|e| e.to_user_issue())
}

#[tauri::command]
pub fn reorder_library_mods(
    app: tauri::AppHandle,
    game_id: String,
    mod_ids: Vec<String>,
) -> Result<GameLibrary, UserFacingIssue> {
    let data = app_data(&app)?;
    crate::library::reorder_mods(&data, &game_id, mod_ids).map_err(|e| e.to_user_issue())
}

#[tauri::command]
pub fn parse_fomod(
    app: tauri::AppHandle,
    game_id: String,
    slug: String,
) -> Result<FomodConfig, UserFacingIssue> {
    let staging = staging_dir_for(&app, &game_id)?;
    ingest::parse_fomod_for_slug(&staging, &slug).map_err(|e| e.to_user_issue())
}

#[tauri::command]
pub fn apply_fomod(
    app: tauri::AppHandle,
    game_id: String,
    mod_id: String,
    slug: String,
    selections: Vec<String>,
) -> Result<IngestedMod, UserFacingIssue> {
    let data = app_data(&app)?;
    let staging = staging_dir_for(&app, &game_id)?;
    let entry = ingest::finalize_fomod_mod(&staging, &mod_id, &slug, &selections)
        .map_err(|e| e.to_user_issue())?;
    persist_ingested(&data, &game_id, std::slice::from_ref(&entry))?;
    Ok(entry)
}

#[tauri::command]
pub fn ingest_mod_paths(
    app: tauri::AppHandle,
    game_id: String,
    paths: Vec<String>,
) -> Result<serde_json::Value, UserFacingIssue> {
    let data = app_data(&app)?;
    let staging = staging_dir_for(&app, &game_id)?;
    let result = ingest::ingest_paths(&staging, &paths).map_err(|e| e.to_user_issue())?;
    persist_ingested(&data, &game_id, &result.mods)?;
    Ok(serde_json::to_value(result).unwrap())
}

#[tauri::command]
pub async fn reinstall_mod(
    app: tauri::AppHandle,
    game_id: String,
    mod_id: String,
) -> Result<IngestedMod, UserFacingIssue> {
    let data = app_data(&app)?;
    let library = crate::library::get_library(&data, &game_id).map_err(|e| e.to_user_issue())?;
    let lib_mod = library
        .mods
        .iter()
        .find(|m| m.id == mod_id)
        .ok_or_else(|| AppError::user("Mod not found in library.").to_user_issue())?
        .clone();

    let staging = staging_dir_for(&app, &game_id)?;
    let entry = ingest::refresh_mod_from_staging(
        &staging,
        &lib_mod.id,
        &lib_mod.slug,
        &lib_mod.name,
        lib_mod.nexus.clone(),
    )
    .map_err(|e| e.to_user_issue())?;
    persist_ingested(&data, &game_id, std::slice::from_ref(&entry))?;
    Ok(entry)
}

#[tauri::command]
pub fn open_staging_folder(app: tauri::AppHandle, game_id: String) -> Result<(), UserFacingIssue> {
    let staging = staging_dir_for(&app, &game_id)?;
    open_path_in_shell(&staging).map_err(|e| e.to_user_issue())
}

#[tauri::command]
pub fn set_mod_notes(
    app: tauri::AppHandle,
    game_id: String,
    mod_id: String,
    notes: Option<String>,
) -> Result<GameLibrary, UserFacingIssue> {
    let data = app_data(&app)?;
    let mut library =
        crate::library::get_library(&data, &game_id).map_err(|e| e.to_user_issue())?;
    let entry = library
        .mods
        .iter_mut()
        .find(|m| m.id == mod_id)
        .ok_or_else(|| AppError::user("Mod not found.").to_user_issue())?;
    entry.notes = notes.filter(|n| !n.trim().is_empty());
    library.updated_at = crate::library::now_ts();
    crate::library::save_library(&data, &library).map_err(|e| e.to_user_issue())?;
    Ok(library)
}

#[tauri::command]
pub fn open_mod_folder(
    app: tauri::AppHandle,
    game_id: String,
    mod_id: String,
    slug: Option<String>,
) -> Result<(), UserFacingIssue> {
    let staging = staging_dir_for(&app, &game_id)?;
    let mod_slug = if let Some(slug) = slug.filter(|s| !s.trim().is_empty()) {
        slug
    } else {
        let data = app_data(&app)?;
        let library =
            crate::library::get_library(&data, &game_id).map_err(|e| e.to_user_issue())?;
        library
            .mods
            .iter()
            .find(|m| m.id == mod_id)
            .map(|m| m.slug.clone())
            .ok_or_else(|| AppError::user("Mod not found in library.").to_user_issue())?
    };
    let mod_root = staging.join(&mod_slug);
    if !mod_root.is_dir() {
        return Err(AppError::user(format!(
            "Mod folder not found at \"{}\".",
            mod_root.display()
        ))
        .to_user_issue());
    }
    open_path_in_shell(&mod_root).map_err(|e| e.to_user_issue())
}
