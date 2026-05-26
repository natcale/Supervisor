/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestTarget {
    pub rel_path: String,
    pub source: String,
    pub mod_id: String,
    pub mod_type: String,
    pub deploy_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployManifest {
    pub game_id: String,
    pub profile_id: String,
    pub staging_path: String,
    pub deploy_method: String,
    pub deployed_at: u64,
    pub targets: Vec<ManifestTarget>,
}

pub fn write_manifest(path: &Path, manifest: &DeployManifest) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(AppError::Io)?;
    }
    let json = serde_json::to_string_pretty(manifest).map_err(AppError::Json)?;
    fs::write(path, json).map_err(AppError::Io)
}

pub fn read_manifest(path: &Path) -> AppResult<Option<DeployManifest>> {
    if !path.is_file() {
        return Ok(None);
    }
    let content = fs::read_to_string(path).map_err(AppError::Io)?;
    Ok(Some(
        serde_json::from_str(&content).map_err(AppError::Json)?,
    ))
}

pub fn manifest_path(app_data: &Path, game_id: &str) -> PathBuf {
    app_data
        .join("deployments")
        .join(game_id)
        .join("supervisor.deployment.json")
}

pub fn state_path(app_data: &Path, game_id: &str) -> PathBuf {
    app_data
        .join("deployments")
        .join(game_id)
        .join("supervisor.state.json")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedDeployState {
    pub manifest: DeployManifest,
    pub report: crate::deploy::verify::DeployReport,
    pub profile_id: String,
    pub profile_name: String,
    pub primary_mod_path: String,
}

pub fn save_state(path: &Path, state: &PersistedDeployState) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(AppError::Io)?;
    }
    let json = serde_json::to_string_pretty(state).map_err(AppError::Json)?;
    fs::write(path, json).map_err(AppError::Io)
}

pub fn load_state(path: &Path) -> AppResult<Option<PersistedDeployState>> {
    if !path.is_file() {
        return Ok(None);
    }
    let content = fs::read_to_string(path).map_err(AppError::Io)?;
    Ok(Some(
        serde_json::from_str(&content).map_err(AppError::Json)?,
    ))
}
