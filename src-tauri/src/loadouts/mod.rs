/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::errors::{AppError, AppResult};
use crate::library::now_ts;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Loadout {
    pub id: String,
    pub name: String,
    pub enabled_mod_ids: Vec<String>,
    pub conflict_resolutions: HashMap<String, String>,
    pub deploy_path_override: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadoutSummary {
    pub id: String,
    pub name: String,
    pub enabled_count: usize,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadoutStore {
    pub game_id: String,
    pub active_loadout_id: String,
    pub loadouts: Vec<Loadout>,
}

fn store_path(app_data: &Path, game_id: &str) -> PathBuf {
    app_data.join("library").join(game_id).join("loadouts.json")
}

pub fn load_store(app_data: &Path, game_id: &str) -> AppResult<LoadoutStore> {
    let path = store_path(app_data, game_id);
    if !path.is_file() {
        let default = default_loadout();
        return Ok(LoadoutStore {
            game_id: game_id.to_string(),
            active_loadout_id: default.id.clone(),
            loadouts: vec![default],
        });
    }
    let raw = fs::read_to_string(&path).map_err(AppError::Io)?;
    serde_json::from_str(&raw).map_err(|e| AppError::user(format!("Corrupt loadouts file: {e}")))
}

pub fn save_store(app_data: &Path, store: &LoadoutStore) -> AppResult<()> {
    let path = store_path(app_data, &store.game_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(AppError::Io)?;
    }
    let raw = serde_json::to_string_pretty(store).map_err(|e| AppError::user(e.to_string()))?;
    fs::write(path, raw).map_err(AppError::Io)
}

fn default_loadout() -> Loadout {
    Loadout {
        id: "default".into(),
        name: "Default".into(),
        enabled_mod_ids: Vec::new(),
        conflict_resolutions: HashMap::new(),
        deploy_path_override: None,
        created_at: now_ts(),
        updated_at: now_ts(),
    }
}

pub fn list_loadouts(app_data: &Path, game_id: &str) -> AppResult<(Vec<LoadoutSummary>, String)> {
    let store = load_store(app_data, game_id)?;
    let summaries = store
        .loadouts
        .iter()
        .map(|l| LoadoutSummary {
            id: l.id.clone(),
            name: l.name.clone(),
            enabled_count: l.enabled_mod_ids.len(),
            updated_at: l.updated_at,
        })
        .collect();
    Ok((summaries, store.active_loadout_id))
}

pub fn get_active_loadout(app_data: &Path, game_id: &str) -> AppResult<Loadout> {
    let store = load_store(app_data, game_id)?;
    store
        .loadouts
        .iter()
        .find(|l| l.id == store.active_loadout_id)
        .cloned()
        .ok_or_else(|| AppError::user("Active loadout not found"))
}

pub fn switch_loadout(app_data: &Path, game_id: &str, loadout_id: &str) -> AppResult<Loadout> {
    let mut store = load_store(app_data, game_id)?;
    if !store.loadouts.iter().any(|l| l.id == loadout_id) {
        return Err(AppError::user("Loadout not found"));
    }
    store.active_loadout_id = loadout_id.to_string();
    let active = store
        .loadouts
        .iter()
        .find(|l| l.id == loadout_id)
        .cloned()
        .unwrap();
    save_store(app_data, &store)?;
    Ok(active)
}

pub fn create_loadout(app_data: &Path, game_id: &str, name: &str) -> AppResult<Loadout> {
    let mut store = load_store(app_data, game_id)?;
    let id = format!("loadout-{}", uuid::Uuid::new_v4());
    let loadout = Loadout {
        id: id.clone(),
        name: name.to_string(),
        enabled_mod_ids: Vec::new(),
        conflict_resolutions: HashMap::new(),
        deploy_path_override: None,
        created_at: now_ts(),
        updated_at: now_ts(),
    };
    store.loadouts.push(loadout.clone());
    save_store(app_data, &store)?;
    Ok(loadout)
}

pub fn update_loadout(app_data: &Path, game_id: &str, mut loadout: Loadout) -> AppResult<Loadout> {
    let mut store = load_store(app_data, game_id)?;
    let Some(existing) = store.loadouts.iter_mut().find(|l| l.id == loadout.id) else {
        return Err(AppError::user("Loadout not found"));
    };
    loadout.updated_at = now_ts();
    *existing = loadout.clone();
    save_store(app_data, &store)?;
    Ok(loadout)
}

pub fn delete_loadout(app_data: &Path, game_id: &str, loadout_id: &str) -> AppResult<()> {
    if loadout_id == "default" {
        return Err(AppError::user("Cannot delete the default loadout"));
    }
    let mut store = load_store(app_data, game_id)?;
    if store.active_loadout_id == loadout_id {
        store.active_loadout_id = "default".into();
    }
    store.loadouts.retain(|l| l.id != loadout_id);
    save_store(app_data, &store)
}

pub fn remove_mod_references(app_data: &Path, game_id: &str, mod_id: &str) -> AppResult<()> {
    let mut store = load_store(app_data, game_id)?;
    let mut changed = false;
    for loadout in &mut store.loadouts {
        let enabled_before = loadout.enabled_mod_ids.len();
        let conflicts_before = loadout.conflict_resolutions.len();
        loadout.enabled_mod_ids.retain(|id| id != mod_id);
        loadout
            .conflict_resolutions
            .retain(|_, winner| winner != mod_id);
        if loadout.enabled_mod_ids.len() != enabled_before
            || loadout.conflict_resolutions.len() != conflicts_before
        {
            loadout.updated_at = now_ts();
            changed = true;
        }
    }
    if changed {
        save_store(app_data, &store)?;
    }
    Ok(())
}
