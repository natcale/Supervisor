/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
//! Loads game profiles from `game_profiles.json` and expands engine templates.
//!
//! Vortex per-game extensions (`extensions/games/*/src/index.ts`) inform mod paths
//! and deploy behavior; this file encodes the same workflows under MIT.

use super::profile::{GameProfile, GameProfileSummary, MergeMode, ModTypeDef, RequirementDef};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProfileEntry {
    id: String,
    name: String,
    #[serde(default)]
    steam_app_ids: Vec<String>,
    #[serde(default)]
    nexus_domains: Vec<String>,
    engine: String,
    #[serde(default)]
    mod_path: Option<String>,
}

struct ProfileStore {
    profiles: Vec<GameProfile>,
    by_id: HashMap<String, usize>,
    by_steam: HashMap<String, usize>,
    by_domain: HashMap<String, usize>,
}

static STORE: OnceLock<ProfileStore> = OnceLock::new();

fn store() -> &'static ProfileStore {
    STORE.get_or_init(load_profiles)
}

fn mt(id: &str, rel_path: &str, merge: bool, priority: u32) -> ModTypeDef {
    ModTypeDef {
        id: id.into(),
        rel_path: rel_path.into(),
        merge,
        priority,
    }
}

fn req(
    id: &str,
    label: &str,
    path: &str,
    optional: bool,
    create_if_missing: bool,
) -> RequirementDef {
    RequirementDef {
        id: id.into(),
        label: label.into(),
        path: path.into(),
        optional,
        create_if_missing,
    }
}

fn template_for(entry: &ProfileEntry) -> (Vec<ModTypeDef>, MergeMode, Vec<RequirementDef>, bool) {
    match entry.engine.as_str() {
        "bethesda" => (
            vec![mt("default", "Data", true, 10), mt("dinput", ".", true, 50)],
            MergeMode::Flat,
            vec![],
            true,
        ),
        "data" => (
            vec![mt("default", "Data", true, 10)],
            MergeMode::Flat,
            vec![],
            false,
        ),
        "cyberpunk" => (
            vec![
                mt("cp77_legacy", "archive/pc/mod", true, 10),
                mt("cp77_redmod", "mods", false, 20),
            ],
            MergeMode::Flat,
            vec![],
            false,
        ),
        "bg3" => (
            vec![
                mt("bg3_pak", "Mods", true, 5),
                mt("bg3_loose", "Data", true, 15),
                mt("bg3_se", ".", true, 50),
            ],
            MergeMode::Flat,
            vec![],
            false,
        ),
        "mods" => (
            vec![mt("default", "Mods", true, 10)],
            MergeMode::Flat,
            vec![],
            false,
        ),
        "mod_root" => (
            vec![mt("default", "mod", true, 10)],
            MergeMode::Flat,
            vec![req("modengine2", "ModEngine2", "mod", true, true)],
            false,
        ),
        "subnautica" => (
            vec![
                mt("bepinex", "BepInEx/plugins", true, 10),
                mt("bepinex_tree", ".", true, 5),
                mt("doorstop", ".", true, 1),
                mt("qmod", "QMods", true, 20),
            ],
            MergeMode::Flat,
            vec![req("bepinex", "BepInEx", "BepInEx", false, false)],
            false,
        ),
        "stardew" => (
            vec![mt("default", "Mods", true, 10), mt("root", ".", true, 5)],
            MergeMode::Flat,
            vec![],
            false,
        ),
        "bepinex" => (
            vec![
                mt("bepinex", "BepInEx/plugins", true, 10),
                mt("bepinex_tree", ".", true, 5),
                mt("doorstop", ".", true, 1),
            ],
            MergeMode::Flat,
            vec![],
            false,
        ),
        "kcd_mods" => (
            vec![mt("default", "Mods", false, 10)],
            MergeMode::PerModFolder,
            vec![],
            false,
        ),
        "marvel_rivals" => {
            let pak_path = entry
                .mod_path
                .as_deref()
                .unwrap_or("MarvelGame/Marvel/Content/Paks/~mods");
            (
                vec![
                    mt("pak", pak_path, true, 5),
                    mt("win64", "MarvelGame/Marvel/Binaries/Win64", false, 10),
                ],
                MergeMode::Flat,
                vec![req(
                    "mods-folder",
                    "Paks ~mods folder",
                    pak_path,
                    false,
                    true,
                )],
                false,
            )
        }
        "unreal_pak" => {
            let path = entry.mod_path.as_deref().unwrap_or("Content/Paks/~mods");
            (
                vec![mt("pak", path, true, 10)],
                MergeMode::Flat,
                vec![req("mods-folder", "Paks ~mods folder", path, false, true)],
                false,
            )
        }
        "mod_path" => {
            let path = entry.mod_path.as_deref().unwrap_or("Mods");
            (
                vec![mt("default", path, true, 10)],
                MergeMode::Flat,
                vec![],
                false,
            )
        }
        "game_root" => (
            vec![mt("default", ".", true, 10)],
            MergeMode::Flat,
            vec![],
            false,
        ),
        other => {
            eprintln!(
                "Unknown profile engine '{other}' for '{}', using Data/",
                entry.id
            );
            (
                vec![mt("default", "Data", true, 10)],
                MergeMode::Flat,
                vec![],
                false,
            )
        }
    }
}

fn expand_entry(entry: ProfileEntry) -> GameProfile {
    let (mod_types, merge_mode, requirements, supports_plugins) = template_for(&entry);
    GameProfile {
        id: entry.id,
        name: entry.name,
        nexus_domains: entry.nexus_domains,
        steam_app_ids: entry.steam_app_ids,
        mod_types,
        merge_mode,
        requirements,
        supports_plugins,
    }
}

fn load_profiles() -> ProfileStore {
    let raw = include_str!("game_profiles.json");
    let entries: Vec<ProfileEntry> =
        serde_json::from_str(raw).expect("game_profiles.json must be valid JSON");
    let mut profiles = Vec::with_capacity(entries.len());
    let mut by_id = HashMap::new();
    let mut by_steam = HashMap::new();
    let mut by_domain = HashMap::new();

    for entry in entries {
        let profile = expand_entry(entry);
        let idx = profiles.len();
        for app_id in &profile.steam_app_ids {
            by_steam.insert(app_id.clone(), idx);
        }
        for domain in &profile.nexus_domains {
            by_domain.insert(domain.to_lowercase(), idx);
        }
        by_id.insert(profile.id.clone(), idx);
        profiles.push(profile);
    }

    ProfileStore {
        profiles,
        by_id,
        by_steam,
        by_domain,
    }
}

pub fn all_profiles() -> &'static [GameProfile] {
    &store().profiles
}

pub fn profile_by_id(id: &str) -> Option<&'static GameProfile> {
    let idx = store().by_id.get(id)?;
    store().profiles.get(*idx)
}

pub fn generic_profile() -> &'static GameProfile {
    profile_by_id("generic-data").expect("generic-data profile must exist in game_profiles.json")
}

pub fn all_profile_summaries() -> Vec<GameProfileSummary> {
    all_profiles()
        .iter()
        .filter(|p| p.id != "generic-data")
        .map(|p| GameProfileSummary {
            id: p.id.clone(),
            name: p.name.clone(),
            primary_mod_path: p.default_mod_type().rel_path.clone(),
            is_generic: false,
            supports_plugins: p.supports_plugins,
            nexus_domains: p.nexus_domains.clone(),
            steam_app_ids: p.steam_app_ids.clone(),
        })
        .collect()
}

pub fn nexus_domain_for_steam(app_id: &str) -> Option<&str> {
    let idx = store().by_steam.get(app_id)?;
    store().profiles[*idx]
        .nexus_domains
        .first()
        .map(|d| d.as_str())
}

pub fn profile_id_for_steam(app_id: &str) -> Option<&str> {
    let idx = store().by_steam.get(app_id)?;
    Some(store().profiles[*idx].id.as_str())
}

pub fn profile_id_for_domain(domain: &str) -> Option<&str> {
    let idx = store().by_domain.get(&domain.to_lowercase())?;
    Some(store().profiles[*idx].id.as_str())
}

/// Best-effort match of a detected game display name to a built-in profile.
pub fn resolve_profile_for_detected_name(name: &str) -> Option<(&'static str, &'static str)> {
    let lower = name.to_lowercase();
    let mut best: Option<(&GameProfile, usize)> = None;
    for profile in all_profiles() {
        if profile.id == "generic-data" {
            continue;
        }
        let pname = profile.name.to_lowercase();
        if lower.contains(&pname) || pname.contains(&lower) {
            let score = pname.len();
            if best.as_ref().map(|(_, s)| *s < score).unwrap_or(true) {
                best = Some((profile, score));
            }
        }
    }
    best.and_then(|(profile, _)| {
        profile
            .nexus_domains
            .first()
            .map(|d| (profile.id.as_str(), d.as_str()))
    })
}

pub fn mod_path_hint(domain: &str) -> Option<&str> {
    profile_id_for_domain(domain)
        .and_then(|id| profile_by_id(id).map(|p| p.default_mod_type().rel_path.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::profile::MergeMode;

    #[test]
    fn loads_all_profiles_including_generic() {
        assert!(all_profiles().len() >= 54);
        assert!(profile_by_id("generic-data").is_some());
    }

    #[test]
    fn bethesda_profiles_use_data_folder_and_plugins() {
        let skyrim = profile_by_id("skyrimse").expect("skyrimse");
        assert!(skyrim.supports_plugins);
        assert!(skyrim.mod_types.iter().any(|t| t.rel_path == "Data"));
    }

    #[test]
    fn kcd_uses_per_mod_folder_mods() {
        let kcd = profile_by_id("kingdomcomdeliverance").expect("kingdomcomdeliverance");
        assert_eq!(kcd.merge_mode, MergeMode::PerModFolder);
        assert_eq!(kcd.default_mod_type().rel_path, "Mods");
    }

    #[test]
    fn steam_lookup_resolves_skyrim_se() {
        assert_eq!(profile_id_for_steam("489830"), Some("skyrimse"));
        assert_eq!(
            nexus_domain_for_steam("489830"),
            Some("skyrimspecialedition")
        );
    }

    #[test]
    fn domain_lookup_is_case_insensitive() {
        assert_eq!(
            profile_id_for_domain("SKYRIMSPECIALEDITION"),
            profile_id_for_domain("skyrimspecialedition")
        );
    }

    #[test]
    fn mod_path_hint_for_cyberpunk() {
        assert_eq!(mod_path_hint("cyberpunk2077"), Some("archive/pc/mod"));
    }

    #[test]
    fn summaries_exclude_generic_profile() {
        let summaries = all_profile_summaries();
        assert!(!summaries.iter().any(|s| s.id == "generic-data"));
        assert!(summaries.len() >= 53);
    }

    #[test]
    fn ready_or_not_uses_paks_mods_folder() {
        let ron = profile_by_id("readyornot").expect("readyornot");
        assert_eq!(
            ron.default_mod_type().rel_path,
            "ReadyOrNot/Content/Paks/~mods"
        );
        assert_eq!(profile_id_for_steam("1144200"), Some("readyornot"));
        assert_eq!(nexus_domain_for_steam("1144200"), Some("readyornot"));
    }

    #[test]
    fn marvel_rivals_supports_pak_and_win64_targets() {
        let mr = profile_by_id("marvelrivals").expect("marvelrivals");
        assert!(mr.mod_types.iter().any(|t| t.id == "pak"));
        assert!(mr.mod_types.iter().any(|t| t.id == "win64"));
        assert_eq!(
            mr.mod_type("win64").map(|t| t.rel_path.as_str()),
            Some("MarvelGame/Marvel/Binaries/Win64")
        );
    }

    #[test]
    fn valheim_supports_bepinex_framework_targets() {
        let valheim = profile_by_id("valheim").expect("valheim");
        assert!(valheim.mod_types.iter().any(|t| t.id == "bepinex_tree"));
        assert!(valheim.mod_types.iter().any(|t| t.id == "doorstop"));
    }

    #[test]
    fn every_profile_has_mod_types() {
        for profile in all_profiles() {
            assert!(
                !profile.mod_types.is_empty(),
                "profile {} must define mod types",
                profile.id
            );
        }
    }
}
