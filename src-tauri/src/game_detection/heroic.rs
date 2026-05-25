/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use crate::errors::AppResult;
use crate::game_detection::types::DetectedGame;
use crate::settings::AppSettings;

pub fn detect_heroic_games(settings: &AppSettings, include_all: bool) -> AppResult<Vec<DetectedGame>> {
    #[cfg(target_os = "linux")]
    {
        detect_heroic_linux(settings, include_all)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (settings, include_all);
        Ok(Vec::new())
    }
}

#[cfg(target_os = "linux")]
fn detect_heroic_linux(settings: &AppSettings, include_all: bool) -> AppResult<Vec<DetectedGame>> {
    use crate::errors::AppError;
    use crate::game_detection::passes_moddable_filter;
    use crate::game_detection::types::GamePlatform;
    use serde::Deserialize;
    use std::fs;
    use std::path::PathBuf;

    #[derive(Debug, Deserialize)]
    struct HeroicConfig {
        games: Option<HeroicGames>,
    }

    #[derive(Debug, Deserialize)]
    struct HeroicGames {
        installed: Option<Vec<HeroicInstalledGame>>,
    }

    #[derive(Debug, Deserialize)]
    struct HeroicInstalledGame {
        app_name: String,
        title: String,
        install_path: String,
        executable: Option<String>,
    }

    let home = std::env::var_os("HOME").map(PathBuf::from);
    let Some(home) = home else {
        return Ok(Vec::new());
    };

    let config_paths = [
        home.join(".config/heroic/config.json"),
        home.join(".config/heroic/store_cache/config.json"),
    ];

    let mut games = Vec::new();
    for config_path in config_paths {
        if !config_path.is_file() {
            continue;
        }
        let content = fs::read_to_string(&config_path).map_err(AppError::Io)?;
        let config: HeroicConfig = serde_json::from_str(&content).map_err(AppError::Json)?;
        if let Some(installed) = config.games.and_then(|g| g.installed) {
            for game in installed {
                let install = PathBuf::from(&game.install_path);
                if !install.is_dir() {
                    continue;
                }
                let install_dir = install
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                if !include_all
                    && !passes_moddable_filter(
                        settings,
                        &GamePlatform::Heroic,
                        Some(&game.app_name),
                        &game.title,
                        &install,
                        install_dir,
                        false,
                        false,
                    )
                {
                    continue;
                }
                games.push(DetectedGame {
                    id: format!("heroic-{}", game.app_name),
                    name: game.title,
                    platform: GamePlatform::Heroic,
                    install_path: game.install_path.clone(),
                    executable: game.executable,
                    app_id: Some(game.app_name),
                    data_path: {
                        let data = install.join("Data");
                        if data.is_dir() {
                            Some(data.to_string_lossy().into_owned())
                        } else {
                            None
                        }
                    },
                    nexus_domain: None,
                    profile_id: None,
                });
            }
        }
    }

    Ok(games)
}
