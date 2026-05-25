/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use crate::errors::{AppError, AppResult};
use crate::game_detection::passes_moddable_filter;
use crate::game_detection::types::{DetectedGame, GamePlatform};
use crate::games::resolve_profile_for_detected_name;
use crate::settings::AppSettings;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct EpicManifest {
    #[serde(rename = "DisplayName")]
    display_name: String,
    #[serde(rename = "InstallLocation")]
    install_location: String,
    #[serde(rename = "AppName")]
    app_name: String,
    #[serde(rename = "LaunchExecutable")]
    launch_executable: Option<String>,
}

pub fn detect_epic_games(settings: &AppSettings, include_all: bool) -> AppResult<Vec<DetectedGame>> {
    let manifests_dir = epic_manifests_dir();
    let Some(dir) = manifests_dir else {
        return Ok(Vec::new());
    };
    if !dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut games = Vec::new();
    for entry in fs::read_dir(&dir).map_err(AppError::Io)? {
        let entry = entry.map_err(AppError::Io)?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("item") {
            continue;
        }
        let content = fs::read_to_string(&path).map_err(AppError::Io)?;
        let manifest: EpicManifest = serde_json::from_str(&content).map_err(AppError::Json)?;
        let install = PathBuf::from(&manifest.install_location);
        if !install.is_dir() {
            continue;
        }

        let install_dir = install
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let known = false;

        if !include_all
            && !passes_moddable_filter(
                settings,
                &GamePlatform::Epic,
                Some(&manifest.app_name),
                &manifest.display_name,
                &install,
                install_dir,
                known,
                false,
            )
        {
            continue;
        }

        let executable = manifest.launch_executable.map(|exe| {
            install.join(exe).to_string_lossy().into_owned()
        });

        let (profile_id, nexus_domain) = resolve_profile_for_detected_name(&manifest.display_name)
            .map(|(id, domain)| (Some(id.to_string()), Some(domain.to_string())))
            .unwrap_or((None, None));

        games.push(DetectedGame {
            id: format!("epic-{}", manifest.app_name),
            name: manifest.display_name,
            platform: GamePlatform::Epic,
            install_path: manifest.install_location.clone(),
            executable,
            app_id: Some(manifest.app_name),
            data_path: infer_data_path(&install),
            nexus_domain,
            profile_id,
        });
    }

    Ok(games)
}

fn infer_data_path(install: &Path) -> Option<String> {
    for sub in ["Data", "Mods", "BepInEx/plugins"] {
        let p = install.join(sub);
        if p.is_dir() {
            return Some(p.to_string_lossy().into_owned());
        }
    }
    None
}

fn epic_manifests_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let program_data = std::env::var_os("ProgramData")?;
        Some(
            PathBuf::from(program_data)
                .join("Epic")
                .join("EpicGamesLauncher")
                .join("Data")
                .join("Manifests"),
        )
    }
    #[cfg(not(windows))]
    {
        None
    }
}
