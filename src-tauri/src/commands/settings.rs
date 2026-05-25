/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::commands::{app_data, configure_runtime_from_settings, open_path_in_shell};
use crate::errors::UserFacingIssue;
use crate::settings::{self, AppPathsInfo, AppSettings};
use std::path::PathBuf;
use tauri::{Emitter, Manager};

#[tauri::command]
pub fn get_app_settings(app: tauri::AppHandle) -> Result<AppSettings, UserFacingIssue> {
    let data = app_data(&app)?;
    settings::load_settings(&data).map_err(|e| e.to_user_issue())
}

#[tauri::command]
pub fn update_app_settings(
    app: tauri::AppHandle,
    settings: AppSettings,
) -> Result<AppSettings, UserFacingIssue> {
    let data = app_data(&app)?;
    settings::save_settings_with_secrets(&data, &settings).map_err(|e| e.to_user_issue())?;
    let saved = settings::load_settings(&data).map_err(|e| e.to_user_issue())?;
    configure_runtime_from_settings(&app, &saved);
    let _ = app.emit("settings://changed", &saved);
    Ok(saved)
}

#[tauri::command]
pub fn get_app_paths(app: tauri::AppHandle) -> Result<AppPathsInfo, UserFacingIssue> {
    let data = app_data(&app)?;
    Ok(settings::app_paths(&data))
}

#[tauri::command]
pub fn open_path(app: tauri::AppHandle, path: String) -> Result<(), UserFacingIssue> {
    let path_buf = PathBuf::from(&path);
    let data = app_data(&app)?;
    settings::ensure_path_for_open(&data, &path_buf).map_err(|e| e.to_user_issue())?;
    open_path_in_shell(&path_buf).map_err(|e| e.to_user_issue())
}

#[tauri::command]
pub fn complete_onboarding(app: tauri::AppHandle) -> Result<(), UserFacingIssue> {
    if let Some(onboarding) = app.get_webview_window("onboarding") {
        let _ = onboarding.close();
    }
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.show();
        let _ = main.set_focus();
    }
    Ok(())
}
