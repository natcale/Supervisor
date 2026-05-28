/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::deploy::manifest::{
    load_state, manifest_path, read_manifest, save_state, state_path, write_manifest,
    DeployManifest, ManifestTarget,
};
use crate::deploy::targets::{apply_target, target_key, target_path};
use crate::errors::{AppError, AppResult};
use crate::hardlink::{remove_managed_link, same_file};
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub(crate) fn remove_targets(targets: &[ManifestTarget]) -> AppResult<usize> {
    let mut removed = 0usize;
    for target in targets {
        if remove_single_target(target, None)? {
            removed += 1;
        }
    }
    Ok(removed)
}

fn remove_single_target(
    target: &ManifestTarget,
    supervisor_staging: Option<&Path>,
) -> AppResult<bool> {
    let deploy_path = target_path(target);
    if !deploy_path.is_file() {
        return Ok(false);
    }
    let source = Path::new(&target.source);
    if remove_managed_link(&deploy_path, source)? {
        return Ok(true);
    }
    if source.exists() {
        if let (Ok(a), Ok(b)) = (deploy_path.canonicalize(), source.canonicalize()) {
            if same_file(&a, &b) {
                std::fs::remove_file(&deploy_path).map_err(AppError::Io)?;
                return Ok(true);
            }
        }
    }
    if let Some(staging) = supervisor_staging {
        if source.starts_with(staging) {
            std::fs::remove_file(&deploy_path).map_err(AppError::Io)?;
            return Ok(true);
        }
    }
    Ok(false)
}

fn cleanup_empty_parents(path: &Path) {
    let mut current = path.parent();
    while let Some(dir) = current {
        if !dir.is_dir() {
            break;
        }
        let empty = std::fs::read_dir(dir)
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(false);
        if !empty {
            break;
        }
        if std::fs::remove_dir(dir).is_err() {
            break;
        }
        current = dir.parent();
    }
}

/// Remove every manifest target using the same rules as single-mod undeploy (staging-aware).
pub(crate) fn remove_all_deployed_targets(
    targets: &[ManifestTarget],
    staging_path: &str,
) -> AppResult<usize> {
    remove_targets_for_mod_removal(targets, staging_path)
}

fn remove_targets_for_mod_removal(
    targets: &[ManifestTarget],
    staging_path: &str,
) -> AppResult<usize> {
    let staging = Path::new(staging_path);
    let mut removed = 0usize;
    for target in targets {
        if remove_single_target(target, Some(staging))? {
            removed += 1;
        }
        cleanup_empty_parents(&target_path(target));
    }
    Ok(removed)
}

pub(crate) fn cleanup_per_mod_folders_from_targets(targets: &[ManifestTarget]) {
    let mut by_root: HashMap<String, HashSet<String>> = HashMap::new();
    for target in targets {
        let Some(folder) = target.rel_path.split('/').next().filter(|s| !s.is_empty()) else {
            continue;
        };
        by_root
            .entry(target.deploy_root.clone())
            .or_default()
            .insert(folder.to_string());
    }

    for (deploy_root, folders) in by_root {
        for folder in folders {
            let path = Path::new(&deploy_root).join(&folder);
            if path.is_dir() {
                let _ = std::fs::remove_dir_all(&path);
                cleanup_empty_parents(&path);
            }
        }
    }
}

pub fn sync_deployment(
    app_data: &Path,
    game_id: &str,
    desired: &[ManifestTarget],
    profile_id: &str,
    staging_path: &str,
) -> AppResult<(Vec<ManifestTarget>, usize, usize)> {
    let path = manifest_path(app_data, game_id);
    let previous = read_manifest(&path)?.unwrap_or(DeployManifest {
        game_id: game_id.into(),
        profile_id: profile_id.into(),
        staging_path: staging_path.into(),
        deploy_method: "hardlink".into(),
        deployed_at: 0,
        targets: vec![],
    });

    let desired_map: HashMap<String, &ManifestTarget> =
        desired.iter().map(|t| (target_key(t), t)).collect();
    let previous_map: HashMap<String, &ManifestTarget> = previous
        .targets
        .iter()
        .map(|t| (target_key(t), t))
        .collect();

    let mut to_remove = Vec::new();
    for (key, old) in &previous_map {
        match desired_map.get(key) {
            None => to_remove.push((*old).clone()),
            Some(new) if old.source != new.source || old.mod_id != new.mod_id => {
                to_remove.push((*old).clone());
            }
            _ => {}
        }
    }

    let removed = remove_targets(&to_remove)?;

    let mut linked = 0usize;
    for target in desired {
        let key = target_key(target);
        let needs_link = match previous_map.get(&key) {
            None => true,
            Some(old) => old.source != target.source || old.mod_id != target.mod_id,
        };
        if needs_link {
            apply_target(target)?;
            linked += 1;
        }
    }

    let deployed_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let manifest = DeployManifest {
        game_id: game_id.to_string(),
        profile_id: profile_id.to_string(),
        staging_path: staging_path.to_string(),
        deploy_method: "hardlink".into(),
        deployed_at,
        targets: desired.to_vec(),
    };
    write_manifest(&path, &manifest)?;

    Ok((manifest.targets, removed, linked))
}

pub fn undeploy_mod_targets(app_data: &Path, game_id: &str, mod_id: &str) -> AppResult<usize> {
    let path = manifest_path(app_data, game_id);
    let Some(mut manifest) = read_manifest(&path)? else {
        return Ok(0);
    };

    let staging_path = manifest.staging_path.clone();
    let (remove, keep): (Vec<_>, Vec<_>) = manifest
        .targets
        .into_iter()
        .partition(|t| t.mod_id == mod_id);
    let removed = remove_targets_for_mod_removal(&remove, &staging_path)?;
    cleanup_per_mod_folders_from_targets(&remove);
    manifest.targets = keep;
    write_manifest(&path, &manifest)?;

    let state_file = state_path(app_data, game_id);
    if let Ok(Some(mut state)) = load_state(&state_file) {
        state.manifest = manifest.clone();
        state.report = crate::deploy::verify::verify_manifest(&state.manifest);
        let _ = save_state(&state_file, &state);
    }

    Ok(removed)
}

/// Drop manifest entries for removed mods or missing staging files, and clean game-folder links.
pub fn prune_deploy_manifest(
    app_data: &Path,
    game_id: &str,
    library: &crate::library::GameLibrary,
) -> AppResult<()> {
    let path = manifest_path(app_data, game_id);
    let Some(mut manifest) = read_manifest(&path)? else {
        return Ok(());
    };

    let mod_ids: HashSet<String> = library.mods.iter().map(|m| m.id.clone()).collect();
    let staging_path = manifest.staging_path.clone();
    let (stale, keep): (Vec<_>, Vec<_>) = manifest
        .targets
        .into_iter()
        .partition(|t| !mod_ids.contains(&t.mod_id) || !Path::new(&t.source).is_file());

    if stale.is_empty() {
        return Ok(());
    }

    remove_targets_for_mod_removal(&stale, &staging_path)?;
    cleanup_per_mod_folders_from_targets(&stale);
    manifest.targets = keep;
    write_manifest(&path, &manifest)?;

    let state_file = state_path(app_data, game_id);
    if let Ok(Some(mut state)) = load_state(&state_file) {
        state.manifest = manifest;
        state.report = crate::deploy::verify::verify_manifest(&state.manifest);
        let _ = save_state(&state_file, &state);
    }

    Ok(())
}

pub fn remove_orphan_redmod_folders(
    game_root: &Path,
    desired_slugs: &HashSet<String>,
) -> AppResult<()> {
    remove_orphan_child_folders(&game_root.join("mods"), desired_slugs)
}

/// Remove per-mod folders under a deploy root that are no longer in the active set.
pub fn remove_orphan_per_mod_folders(
    deploy_root: &Path,
    desired_folders: &HashSet<String>,
) -> AppResult<()> {
    remove_orphan_child_folders(deploy_root, desired_folders)
}

fn remove_orphan_child_folders(parent: &Path, desired: &HashSet<String>) -> AppResult<()> {
    if !parent.is_dir() {
        return Ok(());
    }
    let entries = std::fs::read_dir(parent).map_err(AppError::Io)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if desired.contains(name) {
            continue;
        }
        let _ = std::fs::remove_dir_all(&path);
    }
    Ok(())
}
