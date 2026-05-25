/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use crate::game_detection::GamePlatform;
use std::path::Path;

const BLOCKED_STEAM_APP_IDS: &[&str] = &[
    "228980",  // Steamworks Common Redistributables
    "1070560", // Steam Linux Runtime
    "1391110", // Steam Linux Runtime 3.0
    "1628350", // Steamworks SDK Redist
    "1493710", // Proton Experimental
    "218",     // Source SDK Base 2007
    "223",     // Source SDK Base 2013
    "250820",  // SteamVR
    "228200",  // RPG Maker VX Ace RTP
    "228300",  // GameMaker Studio 1.4
    "229020",  // Unity Player
    "1007",    // Steam Client (legacy)
    "431960",  // Wallpaper Engine
    "753",     // Steam
    "480",     // Spacewar
];

const BLOCKED_KEYWORDS: &[&str] = &[
    "steamworks",
    "redistributable",
    "redist",
    "proton",
    "runtime",
    " sdk",
    "sdk ",
    "dedicated server",
    "toolkit",
    "middleware",
    "soundtrack",
    "playtest server",
    "multiplayer dedicated",
    "server dedicated",
    "beta test",
    "linux runtime",
    "depot",
    "wallpaper engine",
    "benchmark",
    "level editor",
    "map editor",
    "modkit",
    "creation kit",
    "launcher",
    "playtest",
    "utility",
    "tool ",
    " tools",
];

const BLOCKED_DIRS: &[&str] = &[
    "steamworks shared",
    "proton",
    "steamlinuxruntime",
    "commonredist",
    "wallpaper_engine",
];

const MOD_MARKERS: &[&str] = &[
    "Data",
    "data",
    "Mods",
    "mods",
    "BepInEx",
    "QMods",
    "mod",
    "Simple Mod Framework",
    "MarvelGame/Marvel/Content/Paks/~mods",
    "archive/pc/mod",
    "Paks",
    "Content/Paks",
];

pub fn is_blocked_utility(app_id: Option<&str>, name: &str, install_dir: &str) -> bool {
    if let Some(id) = app_id {
        if BLOCKED_STEAM_APP_IDS.contains(&id) {
            return true;
        }
    }

    let name_lower = name.to_lowercase();
    let dir_lower = install_dir.to_lowercase();

    for kw in BLOCKED_KEYWORDS {
        if name_lower.contains(kw) || dir_lower.contains(kw) {
            return true;
        }
    }

    for blocked in BLOCKED_DIRS {
        if dir_lower.contains(blocked) {
            return true;
        }
    }

    false
}

pub fn has_mod_markers(install_path: &Path) -> bool {
    for marker in MOD_MARKERS {
        if install_path.join(marker).is_dir() {
            return true;
        }
    }
    false
}

pub fn is_moddable_game(
    platform: &GamePlatform,
    app_id: Option<&str>,
    name: &str,
    install_path: &Path,
    install_dir: &str,
    known_profile: bool,
    manual: bool,
) -> bool {
    if manual {
        return true;
    }

    if is_blocked_utility(app_id, name, install_dir) {
        return false;
    }

    if known_profile {
        return true;
    }

    if has_mod_markers(install_path) {
        return true;
    }

    // Epic/GOG/Heroic: require mod markers unless profile matched
    match platform {
        GamePlatform::Steam => {
            // Steam: show if mod markers exist (already checked) or profile known
            false
        }
        GamePlatform::Epic | GamePlatform::Gog | GamePlatform::Heroic => false,
        GamePlatform::Manual => true,
    }
}

pub fn is_playable_steam_app(app_id: &str, name: &str, install_dir: &str) -> bool {
    !is_blocked_utility(Some(app_id), name, install_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_detection::GamePlatform;
    use std::path::Path;

    #[test]
    fn blocks_steamworks_redistributables() {
        assert!(is_blocked_utility(
            Some("228980"),
            "Steamworks Common Redistributables",
            "Steamworks Shared"
        ));
    }

    #[test]
    fn blocks_wallpaper_engine_by_name() {
        assert!(is_blocked_utility(None, "Wallpaper Engine", "wallpaper_engine"));
    }

    #[test]
    fn known_profile_is_moddable_without_markers() {
        assert!(is_moddable_game(
            &GamePlatform::Steam,
            Some("489830"),
            "Skyrim Special Edition",
            Path::new("C:/games/skyrim"),
            "Skyrim Special Edition",
            true,
            false,
        ));
    }

    #[test]
    fn unknown_steam_game_needs_markers() {
        assert!(!is_moddable_game(
            &GamePlatform::Steam,
            Some("999999"),
            "Unknown Game",
            Path::new("C:/games/unknown"),
            "Unknown",
            false,
            false,
        ));
    }

    #[test]
    fn manual_games_always_moddable() {
        assert!(is_moddable_game(
            &GamePlatform::Manual,
            None,
            "My Custom Game",
            Path::new("C:/custom"),
            "custom",
            false,
            true,
        ));
    }
}
