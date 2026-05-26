/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::deploy::manifest::{
    load_state, manifest_path, read_manifest, save_state, state_path, DeployManifest,
    PersistedDeployState,
};
use crate::errors::AppResult;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurgeResult {
    pub removed_files: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

pub fn purge_deployment(app_data: &Path, game_id: &str) -> AppResult<PurgeResult> {
    let manifest_file = manifest_path(app_data, game_id);
    let Some(manifest) = read_manifest(&manifest_file)? else {
        return Ok(PurgeResult {
            removed_files: 0,
            skipped: 0,
            errors: vec!["Nothing is deployed for this game.".into()],
        });
    };

    let result = remove_manifest_targets(&manifest)?;

    let state_file = state_path(app_data, game_id);
    let _ = fs::remove_file(&manifest_file);
    let _ = fs::remove_file(&state_file);
    if let Some(parent) = manifest_file.parent() {
        let _ = fs::remove_dir(parent);
    }

    Ok(result)
}

fn remove_manifest_targets(manifest: &DeployManifest) -> AppResult<PurgeResult> {
    let mut removed = 0usize;
    let mut skipped = 0usize;
    let mut errors = Vec::new();

    for target in &manifest.targets {
        let path = Path::new(&target.deploy_root).join(&target.rel_path);
        if !path.exists() {
            skipped += 1;
            continue;
        }
        if !path.is_file() {
            skipped += 1;
            errors.push(format!("Skipped non-file at \"{}\"", path.display()));
            continue;
        }
        match fs::remove_file(&path) {
            Ok(()) => removed += 1,
            Err(e) => errors.push(format!("Could not remove \"{}\": {e}", path.display())),
        }
    }

    Ok(PurgeResult {
        removed_files: removed,
        skipped,
        errors,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployStateResponse {
    pub state: PersistedDeployState,
    pub drift_detected: bool,
    pub checked_at: u64,
}

pub fn refresh_deploy_state(
    app_data: &Path,
    game_id: &str,
) -> AppResult<Option<DeployStateResponse>> {
    let state_file = state_path(app_data, game_id);
    let Some(mut state) = load_state(&state_file)? else {
        return Ok(None);
    };

    let fresh_report = crate::deploy::verify::verify_manifest(&state.manifest);
    let drift_detected =
        !fresh_report.verified || fresh_report.missing > 0 || fresh_report.mismatched > 0;

    state.report = fresh_report;

    let checked_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    save_state(&state_file, &state)?;

    Ok(Some(DeployStateResponse {
        state,
        drift_detected,
        checked_at,
    }))
}
