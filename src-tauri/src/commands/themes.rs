/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::commands::{app_data, open_path_in_shell};
use crate::errors::{AppError, UserFacingIssue};
use crate::settings::{self, AppSettings};
use crate::themes::{
    default_theme, install_theme_archive, list_installed_themes, load_theme_from_dir,
    resolve_theme_dir, LoadedTheme, ThemeSummary,
};
use std::path::PathBuf;
use tauri::Manager;

fn persist_active_theme(
    app_data: &std::path::Path,
    settings: &mut AppSettings,
    theme_id: &str,
) -> Result<(), UserFacingIssue> {
    settings.active_theme_id = theme_id.to_string();
    settings::save_settings(app_data, settings).map_err(|e| e.to_user_issue())
}

fn bundled_themes_dir(app: &tauri::AppHandle) -> Option<PathBuf> {
    if let Ok(resource) = app.path().resource_dir() {
        let bundled = resource.join("themes/bundled");
        if bundled.is_dir() {
            return Some(bundled);
        }
    }

    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../themes/bundled");
    dev.is_dir().then_some(dev)
}

#[tauri::command]
pub fn list_themes(app: tauri::AppHandle) -> Result<Vec<ThemeSummary>, UserFacingIssue> {
    let data = app_data(&app)?;
    let bundled = bundled_themes_dir(&app);
    list_installed_themes(&data, bundled.as_deref()).map_err(|e| e.to_user_issue())
}

#[tauri::command]
pub fn load_active_theme(app: tauri::AppHandle) -> Result<LoadedTheme, UserFacingIssue> {
    let data = app_data(&app)?;
    let bundled = bundled_themes_dir(&app);
    let bundled_root = bundled.as_deref();
    let mut settings = settings::load_settings(&data).map_err(|e| e.to_user_issue())?;
    let requested = settings.active_theme_id.clone();

    match load_theme_from_dir(&data, bundled_root, &requested) {
        Ok(loaded) => Ok(loaded),
        Err(e) if requested != "default" => {
            log::warn!("Active theme \"{requested}\" unavailable: {e}");
            persist_active_theme(&data, &mut settings, "default")?;
            Ok(default_theme())
        }
        Err(e) => Err(e.to_user_issue()),
    }
}

#[tauri::command]
pub fn set_active_theme(
    app: tauri::AppHandle,
    theme_id: String,
) -> Result<LoadedTheme, UserFacingIssue> {
    let data = app_data(&app)?;
    let bundled = bundled_themes_dir(&app);
    let theme_id = theme_id.trim().to_string();
    if theme_id.is_empty() {
        return Err(AppError::user("Theme id cannot be empty.").to_user_issue());
    }

    let loaded =
        load_theme_from_dir(&data, bundled.as_deref(), &theme_id).map_err(|e| e.to_user_issue())?;
    let mut settings = settings::load_settings(&data).map_err(|e| e.to_user_issue())?;
    persist_active_theme(&data, &mut settings, &theme_id)?;
    Ok(loaded)
}

#[tauri::command]
pub fn install_theme(
    app: tauri::AppHandle,
    archive_path: String,
) -> Result<ThemeSummary, UserFacingIssue> {
    let data = app_data(&app)?;
    install_theme_archive(&data, &PathBuf::from(archive_path)).map_err(|e| e.to_user_issue())
}

#[tauri::command]
pub fn get_platform() -> String {
    std::env::consts::OS.to_string()
}

#[tauri::command]
pub fn open_themes_folder(app: tauri::AppHandle) -> Result<(), UserFacingIssue> {
    let data = app_data(&app)?;
    let dir = crate::themes::ensure_themes_dir(&data).map_err(|e| e.to_user_issue())?;
    open_path_in_shell(&dir).map_err(|e| e.to_user_issue())
}

#[tauri::command]
pub fn read_theme_asset(
    app: tauri::AppHandle,
    theme_id: String,
    relative_path: String,
) -> Result<Vec<u8>, UserFacingIssue> {
    let data = app_data(&app)?;
    let bundled = bundled_themes_dir(&app);
    if relative_path.contains("..") {
        return Err(AppError::user("Invalid theme asset path.").to_user_issue());
    }
    let theme_dir =
        resolve_theme_dir(&data, bundled.as_deref(), &theme_id).map_err(|e| e.to_user_issue())?;
    let path = theme_dir.join(&relative_path);
    if !path.is_file() {
        return Err(AppError::user(format!(
            "Theme asset not found: {relative_path}. Reinstall the theme."
        ))
        .to_user_issue());
    }
    std::fs::read(&path).map_err(|e| AppError::Io(e).to_user_issue())
}
