/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
//! Tauri command handlers — thin presentation layer over domain modules.

mod about;
mod collections;
mod deploy;
pub mod downloads;
mod games;
mod library;
mod loadouts;
mod nexus;
mod plugins;
mod settings;
mod themes;

pub use about::*;
pub use collections::*;
pub use deploy::*;
pub use downloads::*;
pub use games::*;
pub use library::*;
pub use loadouts::*;
pub use nexus::*;
pub use plugins::*;
pub use settings::*;
pub use themes::*;

use crate::errors::{AppError, UserFacingIssue};
use crate::ingest::IngestedMod;
use crate::library::GameLibrary;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{Emitter, Manager};

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameStateResponse {
    pub library: GameLibrary,
    pub loadout: crate::loadouts::Loadout,
    pub staging_dir: String,
}

pub(crate) fn app_data(app: &tauri::AppHandle) -> Result<PathBuf, UserFacingIssue> {
    app.path()
        .app_data_dir()
        .map_err(|e| AppError::user(e.to_string()).to_user_issue())
}

pub(crate) fn staging_dir_for(
    app: &tauri::AppHandle,
    game_id: &str,
) -> Result<PathBuf, UserFacingIssue> {
    let data = app_data(app)?;
    let settings = crate::settings::load_settings(&data).map_err(|e| e.to_user_issue())?;
    let dir = crate::settings::game_staging_dir(&data, &settings, game_id);
    std::fs::create_dir_all(&dir).map_err(|e| AppError::Io(e).to_user_issue())?;
    Ok(dir)
}

pub fn configure_runtime_from_settings(
    app: &tauri::AppHandle,
    settings: &crate::settings::AppSettings,
) {
    let enabled = settings.developer_tools;
    for label in ["main", "onboarding"] {
        let Some(window) = app.get_webview_window(label) else {
            continue;
        };
        apply_webview_developer_mode(&window, enabled);
    }
}

fn apply_webview_developer_mode(window: &tauri::WebviewWindow, enabled: bool) {
    let _ = window.with_webview(move |platform| {
        #[cfg(windows)]
        unsafe {
            use webview2_com::Microsoft::Web::WebView2::Win32::{
                ICoreWebView2Controller, ICoreWebView2Settings,
            };
            let controller: ICoreWebView2Controller = platform.controller();
            if let Ok(core) = controller.CoreWebView2() {
                if let Ok(web_settings) = core.Settings() {
                    let settings: ICoreWebView2Settings = web_settings;
                    let _ = settings.SetAreDefaultContextMenusEnabled(enabled);
                }
            }
        }
    });
}

pub(crate) fn persist_ingested(
    app_data: &PathBuf,
    game_id: &str,
    ingested: &[IngestedMod],
) -> Result<GameLibrary, UserFacingIssue> {
    let mut library =
        crate::library::get_library(app_data, game_id).map_err(|e| e.to_user_issue())?;
    for entry in ingested {
        let lib_mod = crate::ingest::ingested_to_library(entry);
        if let Some(existing) = library.mods.iter_mut().find(|m| m.id == lib_mod.id) {
            *existing = lib_mod;
        } else {
            library.mods.push(lib_mod);
        }
    }
    library.updated_at = crate::library::now_ts();
    crate::library::save_library(app_data, &library).map_err(|e| e.to_user_issue())?;

    let mut loadout =
        crate::loadouts::get_active_loadout(app_data, game_id).map_err(|e| e.to_user_issue())?;
    let settings = crate::settings::load_settings(app_data).unwrap_or_default();
    for entry in ingested {
        if entry.install_state == crate::library::InstallState::Installed
            && settings.collections_auto_enable
            && !loadout.enabled_mod_ids.contains(&entry.id)
        {
            loadout.enabled_mod_ids.push(entry.id.clone());
        }
    }
    crate::loadouts::update_loadout(app_data, game_id, loadout).map_err(|e| e.to_user_issue())?;
    Ok(library)
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct NxmDedupKey {
    domain: String,
    mod_id: u64,
    file_id: u64,
}

static NXM_DEDUP: Mutex<Vec<(NxmDedupKey, Instant)>> = Mutex::new(Vec::new());
const NXM_DEDUP_WINDOW: Duration = Duration::from_secs(30);

pub fn handle_nxm_payload(app: &tauri::AppHandle, payload: crate::deep_link::NxmPayload) {
    if let crate::deep_link::NxmPayload::ModDownload(ref link) = payload {
        let key = NxmDedupKey {
            domain: link.game_domain.clone(),
            mod_id: link.mod_id,
            file_id: link.file_id,
        };
        let mut guard = NXM_DEDUP.lock().unwrap();
        let now = Instant::now();
        guard.retain(|(_, t)| now.duration_since(*t) < NXM_DEDUP_WINDOW);
        if guard.iter().any(|(k, _)| k == &key) {
            return;
        }
        guard.push((key, now));
    }
    let _ = app.emit("nxm://received", &payload);
}

pub fn handle_argv(app: &tauri::AppHandle, argv: &[String]) {
    for arg in argv {
        if arg.starts_with("nxm://") {
            if let Some(payload) = crate::deep_link::parse_nxm_url(arg) {
                handle_nxm_payload(app, payload);
            }
        }
    }
}

pub(crate) fn launch_detected_game(
    app: &tauri::AppHandle,
    game: &crate::game_detection::DetectedGame,
) -> crate::errors::AppResult<()> {
    use crate::errors::AppError;
    use crate::game_detection::GamePlatform;
    use crate::games::resolve_profile;
    use std::path::PathBuf;

    let data = app_data(app).map_err(|e| AppError::user(e.title))?;
    let settings = crate::settings::load_settings(&data).unwrap_or_default();
    let profile = resolve_profile(game);
    let game_root = PathBuf::from(&game.install_path);

    if profile.merge_mode == crate::games::MergeMode::Flat
        && profile.id != "generic-data"
        && profile.mod_types.iter().any(|t| t.rel_path == "mod")
    {
        if let Some(path) = settings
            .mod_engine_launcher_path
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            return spawn_launch(PathBuf::from(path), &[]);
        }
    }

    if profile.id == "cyberpunk2077" {
        if manifest_has_redmod(&data, &game.id) {
            if let Some(exe) = crate::cyberpunk::cyberpunk_exe(&game_root) {
                let args = crate::cyberpunk::cyberpunk_launch_args(true);
                return spawn_launch(exe, &args.iter().map(|s| s.as_str()).collect::<Vec<_>>());
            }
        }
    }

    if profile.supports_plugins && settings.prefer_script_extender {
        if let Some(loader) = bethesda_script_extender(&game_root, &profile.id) {
            if loader.is_file() {
                return spawn_launch(loader, &[]);
            }
        }
    }

    if matches!(game.platform, GamePlatform::Steam) {
        if let Some(app_id) = &game.app_id {
            let url = format!("steam://rungameid/{app_id}");
            return open_target(&url);
        }
    }

    if let Some(exe) = &game.executable {
        return open_target(exe);
    }

    Err(AppError::user(
        "Supervisor could not find a way to launch this game.",
    ))
}

fn manifest_has_redmod(app_data: &PathBuf, game_id: &str) -> bool {
    let path = crate::deploy::manifest::manifest_path(app_data, game_id);
    let Ok(Some(manifest)) = crate::deploy::manifest::read_manifest(&path) else {
        return false;
    };
    manifest.targets.iter().any(|t| t.mod_type == "cp77_redmod")
}

fn bethesda_script_extender(game_root: &std::path::Path, profile_id: &str) -> Option<PathBuf> {
    let loader = match profile_id {
        "skyrimse" | "skyrimvr" | "enderalse" => "skse64_loader.exe",
        "skyrim" => "skse_loader.exe",
        "fallout4" | "fallout4vr" => "f4se_loader.exe",
        "starfield" => "sfse_loader.exe",
        "newvegas" => "nvse_loader.exe",
        _ => return None,
    };
    let path = game_root.join(loader);
    if path.is_file() {
        Some(path)
    } else {
        None
    }
}

fn spawn_launch(target: PathBuf, args: &[&str]) -> crate::errors::AppResult<()> {
    use crate::errors::AppError;
    use std::process::Command;
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let mut cmd = Command::new(&target);
        cmd.args(args);
        if let Some(dir) = target.parent() {
            cmd.current_dir(dir);
        }
        cmd.creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(AppError::Io)?;
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        let mut cmd = Command::new(&target);
        cmd.args(args);
        if let Some(dir) = target.parent() {
            cmd.current_dir(dir);
        }
        cmd.spawn().map_err(AppError::Io)?;
        Ok(())
    }
}

pub(crate) fn open_path_in_shell(path: &PathBuf) -> crate::errors::AppResult<()> {
    use crate::errors::AppError;

    if !path.exists() {
        return Err(AppError::user(format!(
            "Path does not exist: {}",
            path.display()
        )));
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let path_arg = path.to_string_lossy().into_owned();
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &path_arg])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(AppError::Io)?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(AppError::Io)?;
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(AppError::Io)?;
        return Ok(());
    }

    #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
    {
        let _ = path;
        Err(AppError::user(
            "Opening folders is not supported on this platform yet.",
        ))
    }
}

pub(crate) fn open_target(target: &str) -> crate::errors::AppResult<()> {
    use crate::errors::AppError;

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        std::process::Command::new("cmd")
            .args(["/C", "start", "", target])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(AppError::Io)?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(target)
            .spawn()
            .map_err(AppError::Io)?;
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(target)
            .spawn()
            .map_err(AppError::Io)?;
        return Ok(());
    }

    #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
    {
        let _ = target;
        Err(AppError::user(
            "Launching games is not supported on this platform yet.",
        ))
    }
}
