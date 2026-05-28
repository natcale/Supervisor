/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::deploy::manifest::{
    load_state, manifest_path, read_manifest, save_state, state_path, DeployManifest,
    PersistedDeployState,
};
use crate::deploy::sync::{cleanup_per_mod_folders_from_targets, remove_all_deployed_targets};
use crate::errors::AppResult;
use crate::games::profile_by_id;
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
    let removed = remove_all_deployed_targets(&manifest.targets, &manifest.staging_path)?;
    let skipped = manifest.targets.len().saturating_sub(removed);

    if profile_by_id(&manifest.profile_id)
        .map(|p| p.merge_mode == crate::games::MergeMode::PerModFolder)
        .unwrap_or(false)
    {
        cleanup_per_mod_folders_from_targets(&manifest.targets);
    }

    Ok(PurgeResult {
        removed_files: removed,
        skipped,
        errors: Vec::new(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deploy::manifest::ManifestTarget;
    use crate::hardlink::hardlink_file;
    use std::fs;

    #[test]
    fn purge_removes_hardlinks_and_per_mod_folder() {
        let base =
            std::env::temp_dir().join(format!("supervisor-purge-test-{}", uuid::Uuid::new_v4()));
        let staging = base.join("staging");
        let game = base.join("game");
        let mods_dir = game.join("Mods");
        let mod_folder = mods_dir.join("Author.TestMod");
        fs::create_dir_all(&staging).unwrap();
        fs::create_dir_all(&mod_folder).unwrap();

        let source = staging.join("manifest.json");
        fs::write(&source, br#"{"id":"Author.TestMod"}"#).unwrap();
        let deployed = mod_folder.join("manifest.json");
        hardlink_file(&source, &deployed).unwrap();

        let manifest = DeployManifest {
            game_id: "kingdomcomdeliverance".into(),
            profile_id: "kingdomcomdeliverance".into(),
            staging_path: staging.to_string_lossy().into_owned(),
            deploy_method: "hardlink".into(),
            deployed_at: 0,
            targets: vec![ManifestTarget {
                rel_path: "Author.TestMod/manifest.json".into(),
                source: source.to_string_lossy().into_owned(),
                mod_id: "mod-1".into(),
                mod_type: "default".into(),
                deploy_root: mods_dir.to_string_lossy().into_owned(),
            }],
        };

        let result = remove_manifest_targets(&manifest).unwrap();
        assert_eq!(result.removed_files, 1);
        assert!(!deployed.is_file());
        assert!(!mod_folder.exists());
        let _ = fs::remove_dir_all(&base);
    }
}
