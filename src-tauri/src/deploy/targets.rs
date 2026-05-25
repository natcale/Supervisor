/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::deploy::manifest::ManifestTarget;
use crate::diagnostics::ModManifest;
use crate::errors::AppResult;
use crate::games::{mod_path_for_type, GameProfile};
use crate::hardlink::hardlink_file;
use crate::install::normalize_mod;
use crate::root_builder::classify_root_files;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub fn target_path(target: &ManifestTarget) -> PathBuf {
    PathBuf::from(&target.deploy_root).join(&target.rel_path)
}

pub fn target_key(target: &ManifestTarget) -> String {
    format!(
        "{}|{}",
        target.deploy_root.replace('\\', "/").to_lowercase(),
        target.rel_path.replace('\\', "/").to_lowercase()
    )
}

pub fn ordered_enabled_ids(mods: &[ModManifest], enabled_ids: &[String]) -> Vec<String> {
    let enabled: HashSet<_> = enabled_ids.iter().collect();
    mods.iter()
        .filter(|m| enabled.contains(&m.id))
        .map(|m| m.id.clone())
        .collect()
}

pub fn compute_desired_targets(
    game_root: &Path,
    staging: &Path,
    profile: &GameProfile,
    mods: &[ModManifest],
    enabled_ids: &[String],
    conflict_resolutions: &HashMap<String, String>,
    deploy_path_override: Option<&str>,
) -> AppResult<Vec<ManifestTarget>> {
    let mod_map: HashMap<_, _> = mods.iter().map(|m| (m.id.clone(), m)).collect();
    let mut all_normalized = Vec::new();
    let mut all_deploy_keys = Vec::new();

    for mod_id in ordered_enabled_ids(mods, enabled_ids) {
        let Some(m) = mod_map.get(&mod_id) else { continue };
        if m.files.is_empty() {
            continue;
        }
        let slug = mod_slug_from_files(&m.files);
        let normalized = normalize_mod(
            staging,
            &mod_id,
            &slug,
            &m.files,
            profile,
            deploy_path_override,
        );
        for file in &normalized.files {
            all_deploy_keys.push((file.deploy_key(), mod_id.clone()));
        }
        all_normalized.push(normalized);
    }

    let winners = build_winner_map(&all_deploy_keys, conflict_resolutions);
    let mut targets = Vec::new();

    for normalized in &all_normalized {
        let mod_type_def = profile
            .mod_type(&normalized.mod_type)
            .unwrap_or_else(|| profile.default_mod_type());
        let deploy_root = if let Some(path) = deploy_path_override.filter(|p| !p.is_empty()) {
            game_root.join(path)
        } else {
            mod_path_for_type(&game_root.to_path_buf(), mod_type_def, &profile.id)
        };
        if mod_type_def.rel_path != "." && mod_type_def.rel_path != "mods" {
            fs::create_dir_all(&deploy_root).map_err(crate::errors::AppError::Io)?;
        }

        let staging_paths: Vec<String> = normalized.files.iter().map(|f| f.source.clone()).collect();
        let (root_entries, _) = classify_root_files(staging, &staging_paths);
        let root_sources: HashSet<String> = root_entries.iter().map(|r| r.source.clone()).collect();

        for file in &normalized.files {
            if !winners
                .get(&file.deploy_key())
                .map(|w| w == &normalized.mod_id)
                .unwrap_or(true)
            {
                continue;
            }

            let source = staging.join(&file.source);

            if root_sources.contains(&file.source) {
                let target_name = Path::new(&file.deploy_rel)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&file.deploy_rel)
                    .to_string();
                targets.push(ManifestTarget {
                    rel_path: target_name,
                    source: source.to_string_lossy().into_owned(),
                    mod_id: normalized.mod_id.clone(),
                    mod_type: "root".into(),
                    deploy_root: game_root.to_string_lossy().into_owned(),
                });
            } else {
                targets.push(ManifestTarget {
                    rel_path: file.deploy_rel.clone(),
                    source: source.to_string_lossy().into_owned(),
                    mod_id: normalized.mod_id.clone(),
                    mod_type: normalized.mod_type.clone(),
                    deploy_root: deploy_root.to_string_lossy().into_owned(),
                });
            }
        }
    }

    Ok(targets)
}

pub fn apply_target(target: &ManifestTarget) -> AppResult<()> {
    let source = PathBuf::from(&target.source);
    let deploy_path = target_path(target);
    if target.mod_type == "root" || target.deploy_root.ends_with('.') {
        // root deploy
    } else if let Some(parent) = deploy_path.parent() {
        fs::create_dir_all(parent).map_err(crate::errors::AppError::Io)?;
    }
    hardlink_file(&source, &deploy_path)
}

fn mod_slug_from_files(files: &[String]) -> String {
    files
        .first()
        .and_then(|f| f.split('/').next())
        .unwrap_or("mod")
        .to_string()
}

fn build_winner_map(
    keys: &[(String, String)],
    resolutions: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut owners: HashMap<String, Vec<String>> = HashMap::new();
    for (key, mod_id) in keys {
        owners.entry(key.clone()).or_default().push(mod_id.clone());
    }

    let mut winners = HashMap::new();
    for (key, mods) in owners {
        if mods.len() == 1 {
            winners.insert(key, mods[0].clone());
            continue;
        }
        if let Some(winner) = resolutions.get(&key) {
            winners.insert(key, winner.clone());
        } else {
            winners.insert(key, mods.last().cloned().unwrap_or_default());
        }
    }
    winners
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_order_last_wins_conflicts() {
        let keys = vec![
            ("textures/a.dds".into(), "mod-a".into()),
            ("textures/a.dds".into(), "mod-b".into()),
        ];
        let winners = build_winner_map(&keys, &HashMap::new());
        assert_eq!(winners.get("textures/a.dds"), Some(&"mod-b".to_string()));
    }

    #[test]
    fn ordered_enabled_follows_library() {
        let mods = vec![
            ModManifest {
                id: "b".into(),
                name: "B".into(),
                files: vec![],
                dependencies: vec![],
            },
            ModManifest {
                id: "a".into(),
                name: "A".into(),
                files: vec![],
                dependencies: vec![],
            },
        ];
        let order = ordered_enabled_ids(&mods, &["a".into(), "b".into()]);
        assert_eq!(order, vec!["b", "a"]);
    }
}
