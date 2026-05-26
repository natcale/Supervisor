/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
mod store;

use crate::errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum InstallState {
    PendingFomod,
    Installed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EndorsementState {
    Undecided,
    Endorsed,
    Abstained,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NexusMeta {
    pub mod_id: u64,
    pub file_id: u64,
    pub domain: String,
    pub version: Option<String>,
    pub author: Option<String>,
    pub picture_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endorsed: Option<EndorsementState>,
    #[serde(default)]
    pub tracked: bool,
    #[serde(default)]
    pub update_available: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryMod {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub files: Vec<String>,
    pub dependencies: Vec<String>,
    pub install_state: InstallState,
    pub installed_at: i64,
    pub nexus: Option<NexusMeta>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameLibrary {
    pub game_id: String,
    pub mods: Vec<LibraryMod>,
    pub updated_at: i64,
}

pub fn get_library(app_data: &Path, game_id: &str) -> AppResult<GameLibrary> {
    store::load_library(app_data, game_id)
}

pub fn save_library(app_data: &Path, library: &GameLibrary) -> AppResult<()> {
    store::save_library(app_data, library)
}

pub fn remove_mod(
    app_data: &Path,
    staging_root: &Path,
    game_id: &str,
    mod_id: &str,
) -> AppResult<GameLibrary> {
    let mut library = store::load_library(app_data, game_id)?;
    let Some(removed_mod) = library.mods.iter().find(|m| m.id == mod_id).cloned() else {
        crate::deploy::prune_deploy_manifest(app_data, game_id, &library)?;
        return Ok(library);
    };

    crate::deploy::undeploy_mod(app_data, game_id, mod_id)?;

    let mod_staging = staging_root.join(&removed_mod.slug);
    if mod_staging.is_dir() {
        fs::remove_dir_all(&mod_staging).map_err(AppError::Io)?;
    }

    crate::loadouts::remove_mod_references(app_data, game_id, mod_id)?;

    library.mods.retain(|m| m.id != mod_id);
    library.updated_at = now_ts();
    store::save_library(app_data, &library)?;

    crate::deploy::prune_deploy_manifest(app_data, game_id, &library)?;

    Ok(library)
}

pub fn reorder_mods(
    app_data: &Path,
    game_id: &str,
    mod_ids: Vec<String>,
) -> AppResult<GameLibrary> {
    let mut library = store::load_library(app_data, game_id)?;
    let mut reordered = Vec::with_capacity(library.mods.len());
    for id in &mod_ids {
        if let Some(entry) = library.mods.iter().find(|m| m.id == *id).cloned() {
            reordered.push(entry);
        }
    }
    for entry in &library.mods {
        if !mod_ids.contains(&entry.id) {
            reordered.push(entry.clone());
        }
    }
    library.mods = reordered;
    library.updated_at = now_ts();
    store::save_library(app_data, &library)?;
    Ok(library)
}

pub fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
