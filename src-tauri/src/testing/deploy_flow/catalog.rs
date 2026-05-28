/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use super::harness::run_profile_flow;
use crate::games::{all_profiles, GameProfile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineKey {
    Bethesda,
    Data,
    Kcd,
    CyberpunkLegacy,
    CyberpunkRedmod,
    Bg3,
    Mods,
    ModRoot,
    Stardew,
    Bepinex,
    Subnautica,
    Marvel,
    UnrealPak,
    ModPath,
    GameRoot,
}

impl EngineKey {
    pub fn needs_requirement_paths(self) -> bool {
        matches!(
            self,
            EngineKey::Marvel | EngineKey::UnrealPak | EngineKey::ModRoot
        )
    }
}

pub fn engine_key(profile: &GameProfile) -> EngineKey {
    if profile.id == "cyberpunk2077" {
        return EngineKey::CyberpunkLegacy;
    }
    if profile.merge_mode == crate::games::MergeMode::PerModFolder {
        return EngineKey::Kcd;
    }
    if profile.supports_plugins {
        return EngineKey::Bethesda;
    }
    if profile.id == "baldursgate3" {
        return EngineKey::Bg3;
    }
    if profile.mod_type("cp77_redmod").is_some() {
        return EngineKey::CyberpunkRedmod;
    }
    if profile.mod_type("cp77_legacy").is_some() {
        return EngineKey::CyberpunkLegacy;
    }
    if profile.mod_type("pak").is_some() && profile.mod_type("win64").is_some() {
        return EngineKey::Marvel;
    }
    if profile.mod_type("pak").is_some() {
        return EngineKey::UnrealPak;
    }
    if profile.mod_type("qmod").is_some() {
        return EngineKey::Subnautica;
    }
    if profile.mod_type("bepinex").is_some() || profile.mod_type("bepinex_tree").is_some() {
        return EngineKey::Bepinex;
    }
    if profile.mod_type("root").is_some() && profile.mod_types.iter().any(|t| t.id == "default") {
        return EngineKey::Stardew;
    }
    if profile.mod_type("default").is_some() && profile.default_mod_type().rel_path == "mod" {
        return EngineKey::ModRoot;
    }
    if profile.default_mod_type().rel_path == "." {
        return EngineKey::GameRoot;
    }
    if profile.default_mod_type().rel_path.eq_ignore_ascii_case("data") {
        return EngineKey::Data;
    }
    if profile.default_mod_type().rel_path.eq_ignore_ascii_case("mods") {
        return EngineKey::Mods;
    }
    EngineKey::ModPath
}

pub fn all_profile_ids() -> Vec<&'static str> {
    all_profiles()
        .iter()
        .filter(|p| p.id != "generic-data")
        .map(|p| p.id.as_str())
        .collect()
}

pub fn run_all() -> Result<(), super::harness::FlowReport> {
    run_matching(|_| true)
}

pub fn run_matching(predicate: impl Fn(&GameProfile) -> bool) -> Result<(), super::harness::FlowReport> {
    let mut failures = Vec::new();
    let mut passed = 0usize;

    for profile in all_profiles() {
        if profile.id == "generic-data" {
            continue;
        }
        if !predicate(profile) {
            continue;
        }
        match run_profile_flow(&profile.id) {
            Ok(()) => passed += 1,
            Err(e) => failures.push(e),
        }
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(super::harness::FlowReport { passed, failures })
    }
}

pub fn run_engine(engine: EngineKey) -> Result<(), super::harness::FlowReport> {
    run_matching(|p| engine_key(p) == engine)
}
