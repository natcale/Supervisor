/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
mod nexus_lookup;
mod profile;
mod profile_loader;

pub use nexus_lookup::*;
pub use profile::*;
pub use profile_loader::{
    all_profile_summaries, all_profiles, generic_profile, profile_by_id,
    resolve_profile_for_detected_name,
};

use crate::game_detection::DetectedGame;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub fn resolve_profile(game: &DetectedGame) -> &'static GameProfile {
    if let Some(id) = game.profile_id.as_deref() {
        if let Some(p) = profile_by_id(id) {
            return p;
        }
    }

    if let Some(app_id) = &game.app_id {
        for profile in all_profiles() {
            if profile.steam_app_ids.iter().any(|a| a == app_id) {
                return profile;
            }
        }
    }

    if let Some(domain) = &game.nexus_domain {
        let needle = domain.to_lowercase();
        for profile in all_profiles() {
            if profile
                .nexus_domains
                .iter()
                .any(|d| d.eq_ignore_ascii_case(&needle))
            {
                return profile;
            }
        }
    }

    generic_profile()
}

pub fn profile_summary(profile: &GameProfile) -> GameProfileSummary {
    GameProfileSummary {
        id: profile.id.clone(),
        name: profile.name.clone(),
        primary_mod_path: profile.default_mod_type().rel_path.clone(),
        is_generic: profile.id == generic_profile().id,
        supports_plugins: profile.supports_plugins,
        nexus_domains: profile.nexus_domains.clone(),
        steam_app_ids: profile.steam_app_ids.clone(),
    }
}

pub fn mod_path_for_type(game_root: &PathBuf, mod_type: &ModTypeDef, profile_id: &str) -> PathBuf {
    if profile_id == "baldursgate3" && mod_type.id == "bg3_pak" {
        if let Some(path) = crate::bg3::bg3_mods_dir() {
            return path;
        }
    }

    if mod_type.rel_path == "." {
        game_root.clone()
    } else {
        game_root.join(&mod_type.rel_path)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployTargetSummary {
    pub id: String,
    pub label: String,
    pub path: String,
}

/// Common folder patterns shown only when auto-detection falls back to generic.
const GENERIC_DEPLOY_TARGETS: &[(&str, &str, &str)] = &[
    ("data", "Data/ (Bethesda, Gamebryo)", "Data"),
    ("mods", "Mods/ (Witcher, Stardew)", "Mods"),
    ("bepinex", "BepInEx/plugins/", "BepInEx/plugins"),
    (
        "pak",
        "Paks/~mods/ (Unreal .pak)",
        "MarvelGame/Marvel/Content/Paks/~mods",
    ),
    ("mod-root", "mod/ (ReShade, Elden Ring)", "mod"),
];

pub fn list_deploy_targets(game: &DetectedGame) -> Vec<DeployTargetSummary> {
    let profile = resolve_profile(game);
    if profile.id != generic_profile().id {
        return Vec::new();
    }

    GENERIC_DEPLOY_TARGETS
        .iter()
        .map(|(id, label, path)| DeployTargetSummary {
            id: (*id).into(),
            label: (*label).into(),
            path: (*path).into(),
        })
        .collect()
}

pub fn attach_profile(game: &mut DetectedGame) {
    let profile = resolve_profile(game);
    game.profile_id = Some(profile.id.clone());
    if profile.id == "baldursgate3" {
        if let Some(path) = crate::bg3::bg3_mods_dir() {
            game.data_path = Some(path.to_string_lossy().into_owned());
            return;
        }
    }

    if game.data_path.is_none() || game.data_path.as_deref() == Some("") {
        let primary = profile.default_mod_type();
        if primary.rel_path != "." {
            game.data_path = Some(
                PathBuf::from(&game.install_path)
                    .join(&primary.rel_path)
                    .to_string_lossy()
                    .into_owned(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_detection::{DetectedGame, GamePlatform};

    fn sample_steam_game(app_id: &str, domain: &str) -> DetectedGame {
        DetectedGame {
            id: format!("steam-{app_id}"),
            name: "Test Game".into(),
            platform: GamePlatform::Steam,
            install_path: "C:/games/test".into(),
            executable: None,
            app_id: Some(app_id.into()),
            data_path: None,
            nexus_domain: Some(domain.into()),
            profile_id: None,
        }
    }

    #[test]
    fn resolve_profile_by_steam_app_id() {
        let mut game = sample_steam_game("489830", "skyrimspecialedition");
        let profile = resolve_profile(&game);
        assert_eq!(profile.id, "skyrimse");
        attach_profile(&mut game);
        assert_eq!(game.profile_id.as_deref(), Some("skyrimse"));
    }

    #[test]
    fn resolve_profile_by_nexus_domain() {
        let game = DetectedGame {
            id: "manual-test".into(),
            name: "Cyberpunk".into(),
            platform: GamePlatform::Manual,
            install_path: "C:/games/cp2077".into(),
            executable: None,
            app_id: None,
            data_path: None,
            nexus_domain: Some("cyberpunk2077".into()),
            profile_id: None,
        };
        let profile = resolve_profile(&game);
        assert_eq!(profile.id, "cyberpunk2077");
    }

    #[test]
    fn unknown_game_falls_back_to_generic() {
        let game = DetectedGame {
            id: "manual-unknown".into(),
            name: "Unknown".into(),
            platform: GamePlatform::Manual,
            install_path: "C:/games/unknown".into(),
            executable: None,
            app_id: None,
            data_path: None,
            nexus_domain: None,
            profile_id: None,
        };
        assert_eq!(resolve_profile(&game).id, generic_profile().id);
    }
}
