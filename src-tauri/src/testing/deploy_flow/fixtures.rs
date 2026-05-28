/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use super::catalog::EngineKey;
use crate::errors::AppResult;
use crate::games::{GameProfile, MergeMode};
use std::fs;
use std::path::Path;

pub struct ModFixture {
    pub mod_id: String,
    pub slug: String,
    pub files: Vec<String>,
}

/// Create requirement paths and minimal game layout before deploy.
pub fn seed_game_tree(game_root: &Path, profile: &GameProfile, engine: EngineKey) -> AppResult<()> {
    for req in &profile.requirements {
        let target = game_root.join(&req.path);
        if req.create_if_missing || engine.needs_requirement_paths() {
            if let Some(parent) = target.parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(parent).map_err(crate::errors::AppError::Io)?;
                }
            }
            if target.extension().is_none() && !target.exists() {
                fs::create_dir_all(&target).map_err(crate::errors::AppError::Io)?;
            }
        }
    }

    match engine {
        EngineKey::Bethesda | EngineKey::Data => {
            fs::create_dir_all(game_root.join("Data")).map_err(crate::errors::AppError::Io)?;
        }
        EngineKey::Kcd => {
            fs::create_dir_all(game_root.join("Mods")).map_err(crate::errors::AppError::Io)?;
        }
        EngineKey::CyberpunkLegacy => {
            fs::create_dir_all(game_root.join("archive/pc/mod"))
                .map_err(crate::errors::AppError::Io)?;
        }
        EngineKey::CyberpunkRedmod => {
            fs::create_dir_all(game_root.join("mods")).map_err(crate::errors::AppError::Io)?;
        }
        EngineKey::Bg3 => {
            fs::create_dir_all(game_root.join("Data")).map_err(crate::errors::AppError::Io)?;
        }
        EngineKey::Bepinex => {
            fs::create_dir_all(game_root.join("BepInEx/plugins"))
                .map_err(crate::errors::AppError::Io)?;
        }
        EngineKey::Subnautica => {
            fs::create_dir_all(game_root.join("QMods")).map_err(crate::errors::AppError::Io)?;
            // Profile lists BepInEx as a hard requirement even for QMod-only deploys.
            fs::create_dir_all(game_root.join("BepInEx")).map_err(crate::errors::AppError::Io)?;
        }
        _ => {}
    }

    Ok(())
}

pub fn write_staging_mod(
    staging: &Path,
    profile: &GameProfile,
    engine: EngineKey,
) -> AppResult<ModFixture> {
    let slug = "supervisor-flow-test";
    let mod_root = staging.join(slug);
    if mod_root.exists() {
        fs::remove_dir_all(&mod_root).map_err(crate::errors::AppError::Io)?;
    }
    fs::create_dir_all(&mod_root).map_err(crate::errors::AppError::Io)?;

    let rel_paths = match engine {
        EngineKey::Bethesda => {
            write_bytes(&mod_root, "Data/SupervisorTest.esp", b"GRUP")?;
            vec!["Data/SupervisorTest.esp"]
        }
        EngineKey::Data => {
            write_bytes(&mod_root, "Meshes/supervisor.nif", b"nif")?;
            vec!["Meshes/supervisor.nif"]
        }
        EngineKey::Kcd => {
            write_bytes(&mod_root, "manifest.json", br#"{"id":"Author.SupervisorFlowTest"}"#)?;
            write_bytes(&mod_root, "supervisor.cfg", b"cfg")?;
            vec!["manifest.json", "supervisor.cfg"]
        }
        EngineKey::CyberpunkLegacy => {
            write_bytes(
                &mod_root,
                "archive/pc/mod/supervisor_flow.archive",
                b"archive",
            )?;
            vec!["archive/pc/mod/supervisor_flow.archive"]
        }
        EngineKey::CyberpunkRedmod => {
            write_bytes(&mod_root, "info.json", br#"{"name":"flow"}"#)?;
            write_bytes(&mod_root, "archives/supervisor.archive", b"archive")?;
            vec!["info.json", "archives/supervisor.archive"]
        }
        EngineKey::Bg3 => {
            write_bytes(&mod_root, "Data/Textures/supervisor.dds", b"dds")?;
            vec!["Data/Textures/supervisor.dds"]
        }
        EngineKey::Mods => {
            write_bytes(&mod_root, "supervisor/content.txt", b"mods")?;
            vec!["supervisor/content.txt"]
        }
        EngineKey::ModRoot => {
            write_bytes(&mod_root, "supervisor.dll", b"dll")?;
            vec!["supervisor.dll"]
        }
        EngineKey::Stardew => {
            write_bytes(&mod_root, "Mods/Supervisor/content.txt", b"x")?;
            write_bytes(&mod_root, "supervisor-root.txt", b"root")?;
            vec!["Mods/Supervisor/content.txt", "supervisor-root.txt"]
        }
        EngineKey::Bepinex => {
            write_bytes(&mod_root, "BepInEx/plugins/SupervisorFlow.dll", b"dll")?;
            vec!["BepInEx/plugins/SupervisorFlow.dll"]
        }
        EngineKey::Subnautica => {
            write_bytes(&mod_root, "QMods/SupervisorFlow/mod.json", br#"{"name":"flow"}"#)?;
            vec!["QMods/SupervisorFlow/mod.json"]
        }
        EngineKey::Marvel => {
            write_bytes(&mod_root, "skin.pak", b"pak")?;
            vec!["skin.pak"]
        }
        EngineKey::UnrealPak => {
            write_bytes(&mod_root, "supervisor_flow.pak", b"pak")?;
            vec!["supervisor_flow.pak"]
        }
        EngineKey::ModPath | EngineKey::GameRoot => {
            write_bytes(&mod_root, "supervisor_flow.dat", b"dat")?;
            vec!["supervisor_flow.dat"]
        }
    };

  let files: Vec<String> = rel_paths
        .into_iter()
        .map(|rel| format!("{slug}/{rel}"))
        .collect();

    let mod_id = format!("flow-{}", profile.id);
    Ok(ModFixture {
        mod_id,
        slug: slug.into(),
        files,
    })
}

fn write_bytes(root: &Path, rel: &str, data: &[u8]) -> AppResult<()> {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(crate::errors::AppError::Io)?;
    }
    fs::write(path, data).map_err(crate::errors::AppError::Io)?;
    Ok(())
}

pub fn per_mod_folder_name(profile: &GameProfile, staging: &Path, fixture: &ModFixture) -> Option<String> {
    if profile.merge_mode != MergeMode::PerModFolder {
        return None;
    }
    let mod_root = staging.join(&fixture.slug);
    Some(crate::install::per_mod_deploy_folder(
        &mod_root,
        &fixture.slug,
    ))
}
