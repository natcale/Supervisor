/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use crate::errors::AppResult;
use crate::game_detection::passes_moddable_filter;
use crate::game_detection::types::{DetectedGame, GamePlatform};
use crate::games::resolve_profile_for_detected_name;
use crate::settings::AppSettings;
use std::path::{Path, PathBuf};

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

pub fn detect_gog_games(settings: &AppSettings, include_all: bool) -> AppResult<Vec<DetectedGame>> {
    #[cfg(windows)]
    {
        detect_gog_windows(settings, include_all)
    }
    #[cfg(not(windows))]
    {
        Ok(Vec::new())
    }
}

#[cfg(windows)]
fn detect_gog_windows(settings: &AppSettings, include_all: bool) -> AppResult<Vec<DetectedGame>> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let gog_key = hklm.open_subkey("SOFTWARE\\GOG.com\\Games");
    let Ok(gog_key) = gog_key else {
        return Ok(Vec::new());
    };

    let mut games = Vec::new();
    for name in gog_key.enum_keys().flatten() {
        if let Ok(sub) = gog_key.open_subkey(&name) {
            let game_name: String = sub.get_value("gameName").unwrap_or_else(|_| name.clone());
            let path: String = sub.get_value("path").unwrap_or_default();
            if path.is_empty() {
                continue;
            }
            let install = PathBuf::from(&path);
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
                    &GamePlatform::Gog,
                    Some(&name),
                    &game_name,
                    &install,
                    install_dir,
                    false,
                    false,
                )
            {
                continue;
            }
            let exe: String = sub.get_value("exe").unwrap_or_default();
            let (profile_id, nexus_domain) = resolve_profile_for_detected_name(&game_name)
                .map(|(id, domain)| (Some(id.to_string()), Some(domain.to_string())))
                .unwrap_or((None, None));
            games.push(DetectedGame {
                id: format!("gog-{name}"),
                name: game_name,
                platform: GamePlatform::Gog,
                install_path: path.clone(),
                executable: if exe.is_empty() {
                    None
                } else {
                    Some(install.join(exe).to_string_lossy().into_owned())
                },
                app_id: Some(name),
                data_path: data_path_for(&install),
                nexus_domain,
                profile_id,
            });
        }
    }

    Ok(games)
}

fn data_path_for(install: &Path) -> Option<String> {
    for sub in ["Data", "Mods"] {
        let p = install.join(sub);
        if p.is_dir() {
            return Some(p.to_string_lossy().into_owned());
        }
    }
    None
}
