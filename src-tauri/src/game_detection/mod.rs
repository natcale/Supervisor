/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

mod filter;
mod epic;
mod gog;
mod heroic;
pub mod manual;
mod steam;
mod types;

use crate::errors::AppResult;
use crate::settings::{load_settings, AppSettings};
use std::time::{SystemTime, UNIX_EPOCH};

pub use types::*;

pub fn scan_all_games(app_data: &std::path::Path, include_all: bool) -> AppResult<GameScanResult> {
    let settings = load_settings(app_data).unwrap_or_default();
    let mut games = Vec::new();

    if settings.scan_steam {
        games.extend(steam::detect_steam_games(&settings, include_all)?);
    }
    if settings.scan_epic {
        games.extend(epic::detect_epic_games(&settings, include_all)?);
    }
    if settings.scan_gog {
        games.extend(gog::detect_gog_games(&settings, include_all)?);
    }
    if settings.scan_heroic {
        games.extend(heroic::detect_heroic_games(&settings, include_all)?);
    }
    games.extend(manual::load_manual_games(app_data)?);

    for game in &mut games {
        crate::games::attach_profile(game);
    }

    games.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    let mut seen = std::collections::HashSet::new();
    games.retain(|g| seen.insert(g.id.clone()));

    let scanned_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Ok(GameScanResult { games, scanned_at })
}

pub(crate) fn passes_moddable_filter(
    _settings: &AppSettings,
    platform: &GamePlatform,
    app_id: Option<&str>,
    name: &str,
    install_path: &std::path::Path,
    install_dir: &str,
    known_profile: bool,
    manual: bool,
) -> bool {
    filter::is_moddable_game(
        platform,
        app_id,
        name,
        install_path,
        install_dir,
        known_profile,
        manual,
    )
}
