/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use crate::errors::{AppError, AppResult};
use crate::game_detection::filter::{has_mod_markers, is_playable_steam_app};
use crate::game_detection::types::{DetectedGame, GamePlatform};
use crate::game_detection::passes_moddable_filter;
use crate::games::{nexus_domain_for_steam, profile_id_for_steam};
use crate::settings::AppSettings;
use crate::vdf::{find_string, parse_vdf, VdfValue};
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

pub fn detect_steam_games(settings: &AppSettings, include_all: bool) -> AppResult<Vec<DetectedGame>> {
    let mut libraries = Vec::new();

    #[cfg(windows)]
    {
        if let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER).open_subkey("Software\\Valve\\Steam") {
            if let Ok(path) = hkcu.get_value::<String, _>("SteamPath") {
                libraries.push(PathBuf::from(path.replace('/', "\\")));
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(home) = dirs_home() {
            libraries.push(home.join(".steam/steam"));
            libraries.push(home.join(".local/share/Steam"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs_home() {
            libraries.push(home.join("Library/Application Support/Steam"));
        }
    }

    let mut library_paths = Vec::new();
    for root in libraries {
        if root.exists() {
            library_paths.push(root.clone());
            let vdf_path = root.join("steamapps/libraryfolders.vdf");
            if vdf_path.exists() {
                if let Ok(content) = fs::read_to_string(&vdf_path) {
                    library_paths.extend(parse_library_folders(&content));
                }
            }
            let legacy = root.join("SteamApps/libraryfolders.vdf");
            if legacy.exists() {
                if let Ok(content) = fs::read_to_string(&legacy) {
                    library_paths.extend(parse_library_folders(&content));
                }
            }
        }
    }

    library_paths.sort();
    library_paths.dedup();

    let mut detected = Vec::new();
    let mut seen_apps = std::collections::HashSet::new();
    for library in library_paths {
        let steamapps = library.join("steamapps");
        if !steamapps.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&steamapps).map_err(AppError::Io)? {
            let entry = entry.map_err(AppError::Io)?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("acf") {
                continue;
            }
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if !file_name.starts_with("appmanifest_") {
                    continue;
                }
            }
            if let Ok(game) = parse_app_manifest(&path, settings, include_all) {
                if seen_apps.insert(game.id.clone()) {
                    detected.push(game);
                }
            }
        }
    }

    Ok(detected)
}

fn parse_library_folders(content: &str) -> Vec<PathBuf> {
    let Ok(parsed) = parse_vdf(content) else {
        return Vec::new();
    };
    let Some(folders) = parsed.first().and_then(|e| {
        if let VdfValue::Object(ref obj) = e.value {
            Some(obj.as_slice())
        } else {
            None
        }
    }) else {
        return Vec::new();
    };

    let mut paths = Vec::new();
    for entry in folders {
        if let VdfValue::Object(ref folder) = entry.value {
            if let Some(path) = find_string(folder, "path") {
                paths.push(PathBuf::from(path.replace("\\\\", "\\")));
            }
        }
    }
    paths
}

fn parse_app_manifest(path: &Path, settings: &AppSettings, include_all: bool) -> AppResult<DetectedGame> {
    let content = fs::read_to_string(path).map_err(AppError::Io)?;
    let parsed = parse_vdf(&content).map_err(|e| AppError::user(format!("Invalid manifest: {e}")))?;

    let root = parsed
        .first()
        .and_then(|e| {
            if let VdfValue::Object(ref obj) = e.value {
                Some(obj.as_slice())
            } else {
                None
            }
        })
        .ok_or_else(|| AppError::user("Manifest missing AppState"))?;

    let app_id = find_string(root, "appid").unwrap_or_default().to_string();
    let name = find_string(root, "name").unwrap_or("Unknown Steam Game").to_string();
    let install_dir = find_string(root, "installdir").unwrap_or_default();
    let state_flags = find_string(root, "StateFlags").unwrap_or("0");
    if state_flags != "4" {
        return Err(AppError::user("Game not fully installed"));
    }

    let steamapps = path.parent().ok_or_else(|| AppError::user("Invalid path"))?;
    let install_path = steamapps.join("common").join(&install_dir);
    if !install_path.is_dir() {
        return Err(AppError::user("Install folder missing"));
    }

    if !include_all {
        if !is_playable_steam_app(&app_id, &name, &install_dir) {
            return Err(AppError::user("Not a playable game"));
        }

        let known_profile = profile_id_for_steam(&app_id).is_some();
        if !passes_moddable_filter(
            settings,
            &GamePlatform::Steam,
            Some(&app_id),
            &name,
            &install_path,
            &install_dir,
            known_profile,
            false,
        ) {
            return Err(AppError::user("Not a moddable game"));
        }
    }

    let nexus_domain = nexus_domain_for_steam(&app_id).map(str::to_string);
    let executable = find_game_executable(&install_path);

    Ok(DetectedGame {
        id: format!("steam-{app_id}"),
        name,
        platform: GamePlatform::Steam,
        install_path: install_path.to_string_lossy().into_owned(),
        executable: executable.map(|p| p.to_string_lossy().into_owned()),
        app_id: Some(app_id),
        data_path: infer_data_path(&install_path, nexus_domain.as_deref()),
        nexus_domain,
        profile_id: None,
    })
}

fn find_game_executable(install_path: &Path) -> Option<PathBuf> {
    if let Ok(entries) = fs::read_dir(install_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if path.extension().and_then(|e| e.to_str()) == Some("exe") {
                    if path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| !n.starts_with("unins"))
                        .unwrap_or(false)
                    {
                        return Some(path);
                    }
                }
            }
        }
    }
    None
}

fn infer_data_path(install_path: &Path, domain: Option<&str>) -> Option<String> {
    if let Some(d) = domain {
        if let Some(sub) = crate::games::mod_path_hint(d) {
            return Some(install_path.join(sub).to_string_lossy().into_owned());
        }
    }

    if has_mod_markers(install_path) {
        for sub in [
            "Data",
            "Mods",
            "BepInEx/plugins",
            "mod",
            "Simple Mod Framework/Mods",
        ] {
            let p = install_path.join(sub);
            if p.is_dir() {
                return Some(p.to_string_lossy().into_owned());
            }
        }
    }

    Some(install_path.join("Data").to_string_lossy().into_owned())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}
