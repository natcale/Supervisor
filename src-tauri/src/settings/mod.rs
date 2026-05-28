/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::errors::{AppError, AppResult};
use crate::themes;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum UpdateCheckMode {
    Manual,
    OnRefresh,
    OnStartup,
}

impl Default for UpdateCheckMode {
    fn default() -> Self {
        Self::OnRefresh
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    #[serde(default)]
    pub onboarding_complete: bool,
    pub update_check_mode: UpdateCheckMode,
    /// Display-only for now; deploy engine uses hardlinks.
    pub deploy_method: String,
    #[serde(default = "default_true")]
    pub auto_deploy_on_change: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loot_path: Option<String>,

    // Game detection
    #[serde(default = "default_true")]
    pub scan_steam: bool,
    #[serde(default = "default_true")]
    pub scan_epic: bool,
    #[serde(default = "default_true")]
    pub scan_gog: bool,
    #[serde(default = "default_true")]
    pub scan_heroic: bool,
    #[serde(default)]
    pub show_unmoddable_games: bool,

    // Downloads
    #[serde(default = "default_max_downloads")]
    pub max_concurrent_downloads: u32,
    #[serde(default = "default_true")]
    pub auto_start_downloads: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub download_speed_limit_kbps: Option<u32>,

    // Deployment
    #[serde(default)]
    pub auto_purge_before_deploy: bool,
    #[serde(default)]
    pub confirm_before_deploy: bool,
    #[serde(default = "default_true")]
    pub verify_after_deploy: bool,
    #[serde(default = "default_true")]
    pub auto_sort_plugins: bool,

    // UI
    #[serde(default = "default_true")]
    pub show_profile_warnings: bool,
    #[serde(default)]
    pub compact_mod_list: bool,
    #[serde(default = "default_true")]
    pub remember_last_game: bool,
    #[serde(default)]
    pub always_show_plugins: bool,
    #[serde(default)]
    pub developer_tools: bool,
    #[serde(default)]
    pub compact_game_sidebar: bool,
    #[serde(default)]
    pub compact_game_sidebar_hidden: bool,
    #[serde(default = "default_true")]
    pub show_status_bar: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_game_id: Option<String>,
    #[serde(default = "default_theme_id")]
    pub active_theme_id: String,

    /// Populated at runtime from the OS keyring; never written to settings.json.
    #[serde(default, skip_serializing)]
    pub nexus_api_key: Option<String>,
    #[serde(default, skip_serializing)]
    pub has_nexus_api_key: bool,

    // Collections
    #[serde(default)]
    pub collections_skip_optional: bool,
    #[serde(default = "default_true")]
    pub collections_auto_enable: bool,

    #[serde(default = "default_true")]
    pub prefer_script_extender: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mod_engine_launcher_path: Option<String>,

    // Advanced
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub staging_root_override: Option<String>,
    #[serde(default)]
    pub debug_logging: bool,
    #[serde(default)]
    pub ignore_deploy_requirements: bool,
}

fn default_theme_id() -> String {
    "default".into()
}

fn default_max_downloads() -> u32 {
    2
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            onboarding_complete: false,
            update_check_mode: UpdateCheckMode::OnRefresh,
            deploy_method: "hardlink".into(),
            auto_deploy_on_change: true,
            loot_path: None,
            scan_steam: true,
            scan_epic: true,
            scan_gog: true,
            scan_heroic: true,
            show_unmoddable_games: false,
            max_concurrent_downloads: 2,
            auto_start_downloads: true,
            download_speed_limit_kbps: None,
            auto_purge_before_deploy: false,
            confirm_before_deploy: false,
            verify_after_deploy: true,
            auto_sort_plugins: true,
            show_profile_warnings: true,
            compact_mod_list: false,
            remember_last_game: true,
            always_show_plugins: false,
            developer_tools: false,
            compact_game_sidebar: false,
            compact_game_sidebar_hidden: false,
            show_status_bar: true,
            last_game_id: None,
            active_theme_id: default_theme_id(),
            nexus_api_key: None,
            has_nexus_api_key: false,
            collections_skip_optional: false,
            collections_auto_enable: true,
            prefer_script_extender: true,
            mod_engine_launcher_path: None,
            staging_root_override: None,
            debug_logging: false,
            ignore_deploy_requirements: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppPathsInfo {
    pub app_data_dir: String,
    pub staging_root: String,
    pub downloads_dir: String,
    pub themes_dir: String,
}

fn settings_path(app_data: &Path) -> PathBuf {
    app_data.join("settings.json")
}

pub fn load_settings(app_data: &Path) -> AppResult<AppSettings> {
    let path = settings_path(app_data);
    let mut settings = if !path.is_file() {
        AppSettings::default()
    } else {
        let raw = fs::read_to_string(&path).map_err(AppError::Io)?;
        serde_json::from_str(&raw).map_err(|e| AppError::user(format!("Corrupt settings: {e}")))?
    };
    populate_runtime_settings(&mut settings);
    Ok(settings)
}

fn populate_runtime_settings(settings: &mut AppSettings) {
    settings.has_nexus_api_key = crate::secrets::has_nexus_api_key();
    settings.nexus_api_key = crate::secrets::get_nexus_api_key().ok().flatten();
}

pub fn save_settings(app_data: &Path, settings: &AppSettings) -> AppResult<()> {
    fs::create_dir_all(app_data).map_err(AppError::Io)?;
    let mut to_save = settings.clone();
    to_save.nexus_api_key = None;
    to_save.has_nexus_api_key = false;
    let raw = serde_json::to_string_pretty(&to_save).map_err(|e| AppError::user(e.to_string()))?;
    fs::write(settings_path(app_data), raw).map_err(AppError::Io)
}

pub fn save_settings_with_secrets(app_data: &Path, settings: &AppSettings) -> AppResult<()> {
    if let Some(ref key) = settings.nexus_api_key {
        if key.trim().is_empty() {
            crate::secrets::delete_nexus_api_key()?;
        } else {
            crate::secrets::set_nexus_api_key(key)?;
        }
    }
    save_settings(app_data, settings)
}

pub fn game_staging_dir(app_data: &Path, settings: &AppSettings, game_id: &str) -> PathBuf {
    staging_root(app_data, settings).join(game_id)
}

pub fn staging_root(app_data: &Path, settings: &AppSettings) -> PathBuf {
    if let Some(override_path) = settings
        .staging_root_override
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        return PathBuf::from(override_path);
    }
    app_data.join("staging")
}

pub fn ensure_app_dirs(app_data: &Path) -> AppResult<()> {
    fs::create_dir_all(app_data).map_err(AppError::Io)?;
    themes::ensure_themes_dir(app_data)?;
    fs::create_dir_all(app_data.join("downloads")).map_err(AppError::Io)?;
    let settings = load_settings(app_data).unwrap_or_default();
    fs::create_dir_all(staging_root(app_data, &settings)).map_err(AppError::Io)?;
    Ok(())
}

pub fn ensure_path_for_open(app_data: &Path, target: &Path) -> AppResult<()> {
    if target.exists() {
        return Ok(());
    }

    let _ = ensure_app_dirs(app_data);

    if target.is_dir() || target.extension().is_none() {
        fs::create_dir_all(target).map_err(AppError::Io)?;
    }

    Ok(())
}

pub fn app_paths(app_data: &Path) -> AppPathsInfo {
    let _ = ensure_app_dirs(app_data);
    let settings = load_settings(app_data).unwrap_or_default();
    let themes_dir = themes::themes_dir(app_data);
    AppPathsInfo {
        app_data_dir: app_data.to_string_lossy().into_owned(),
        staging_root: staging_root(app_data, &settings)
            .to_string_lossy()
            .into_owned(),
        downloads_dir: app_data.join("downloads").to_string_lossy().into_owned(),
        themes_dir: themes_dir.to_string_lossy().into_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn default_settings_roundtrip() {
        let dir = std::env::temp_dir().join(format!("supervisor-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let defaults = AppSettings::default();
        save_settings(&dir, &defaults).unwrap();
        let loaded = load_settings(&dir).unwrap();
        assert_eq!(loaded.scan_steam, defaults.scan_steam);
        assert_eq!(loaded.max_concurrent_downloads, 2);
        assert_eq!(loaded.deploy_method, "hardlink");
        assert!(!loaded.onboarding_complete);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn partial_json_gets_defaults() {
        let dir = std::env::temp_dir().join(format!("supervisor-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("settings.json"), r#"{"deployMethod":"hardlink"}"#).unwrap();
        let loaded = load_settings(&dir).unwrap();
        assert!(!loaded.onboarding_complete);
        assert!(loaded.scan_steam);
        assert_eq!(loaded.max_concurrent_downloads, 2);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn saves_onboarding_completion() {
        let dir = std::env::temp_dir().join(format!("supervisor-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let settings = AppSettings {
            onboarding_complete: true,
            ..AppSettings::default()
        };
        save_settings(&dir, &settings).unwrap();
        let loaded = load_settings(&dir).unwrap();
        assert!(loaded.onboarding_complete);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn staging_override_takes_precedence() {
        let settings = AppSettings {
            staging_root_override: Some("D:\\Staging".into()),
            ..AppSettings::default()
        };
        let root = staging_root(Path::new("C:\\AppData"), &settings);
        assert_eq!(root, PathBuf::from("D:\\Staging"));
    }
}
