/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::deploy::manifest::{
    PersistedDeployState, save_state, state_path,
};
use crate::deploy::requirements::{check_requirements, profile_mismatch_warnings};
use crate::deploy::sync::{remove_orphan_redmod_folders, sync_deployment, undeploy_mod_targets};
use crate::deploy::targets::{compute_desired_targets, ordered_enabled_ids};
use crate::deploy::verify::{verify_manifest, DeployReport};
use crate::diagnostics::ModManifest;
use crate::errors::{AppError, AppResult};
use crate::games::{
    generic_profile, profile_by_id, GameProfile,
};
use crate::hardlink::check_same_partition;
use crate::install::normalize_mod;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployGameRequest {
    pub game_id: String,
    pub game_dir: String,
    pub profile_id: Option<String>,
    pub staging_dir: String,
    pub mods: Vec<ModManifest>,
    pub enabled_ids: Vec<String>,
    pub conflict_resolutions: HashMap<String, String>,
    pub ignore_requirements: bool,
    pub deploy_path_override: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployResult {
    pub manifest: crate::deploy::manifest::DeployManifest,
    pub report: DeployReport,
    pub summary: String,
    pub deployed_files: usize,
    pub profile_id: String,
    pub profile_name: String,
    pub primary_mod_path: String,
}

pub fn deploy_game(app_data: &Path, request: &DeployGameRequest, verify_after: bool) -> AppResult<DeployResult> {
    let game_root = PathBuf::from(&request.game_dir);
    let staging = PathBuf::from(&request.staging_dir);

    let profile = request
        .profile_id
        .as_deref()
        .and_then(profile_by_id)
        .unwrap_or_else(|| resolve_profile_for_request(request));

    let partition = check_same_partition(&staging, &game_root)?;
    if !partition.same_partition {
        return Err(AppError::user(
            partition
                .guidance
                .map(|g| g.explanation)
                .unwrap_or_else(|| "Staging and game folders must be on the same drive.".into()),
        ));
    }

    if !request.ignore_requirements {
        let req_issues: Vec<_> = check_requirements(&game_root, profile)
            .into_iter()
            .filter(|i| i.id.starts_with("req-"))
            .collect();
        if let Some(issue) = req_issues.into_iter().next() {
            return Err(AppError::user(format!("{} {}", issue.title, issue.explanation)));
        }
    }

    let desired = compute_desired_targets(
        &game_root,
        &staging,
        profile,
        &request.mods,
        &request.enabled_ids,
        &request.conflict_resolutions,
        request.deploy_path_override.as_deref(),
    )?;

    let (targets, removed, linked) = sync_deployment(
        app_data,
        &request.game_id,
        &desired,
        profile.id.as_str(),
        &request.staging_dir,
    )?;

    let manifest = crate::deploy::manifest::DeployManifest {
        game_id: request.game_id.clone(),
        profile_id: profile.id.clone(),
        staging_path: request.staging_dir.clone(),
        deploy_method: "hardlink".into(),
        deployed_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        targets: targets.clone(),
    };

    run_post_deploy_hooks(profile, &game_root, &staging, request)?;

    let primary_path = request
        .deploy_path_override
        .clone()
        .unwrap_or_else(|| profile.default_mod_type().rel_path.to_string());

    let deployed_count = targets.len();
    let mut report = if verify_after {
        verify_manifest(&manifest)
    } else {
        DeployReport {
            verified: true,
            linked: deployed_count,
            missing: 0,
            mismatched: 0,
            issues: Vec::new(),
            profile_warning: None,
        }
    };
    if profile.id == "generic-data" && request.deploy_path_override.is_none() {
        report.profile_warning = Some(
            "Deployed using generic Data/ folder — verify this is correct for your game.".into(),
        );
    }

    let summary = if removed > 0 || linked > 0 {
        format!(
            "Updated deployment: {linked} linked, {removed} removed ({} active).",
            deployed_count
        )
    } else {
        build_summary(&report, profile, deployed_count)
    };

    let state = PersistedDeployState {
        manifest: manifest.clone(),
        report: report.clone(),
        profile_id: profile.id.to_string(),
        profile_name: profile.name.to_string(),
        primary_mod_path: primary_path.clone(),
    };

    save_state(&state_path(app_data, &request.game_id), &state)?;

    Ok(DeployResult {
        manifest,
        report,
        summary,
        deployed_files: deployed_count,
        profile_id: profile.id.to_string(),
        profile_name: profile.name.to_string(),
        primary_mod_path: primary_path,
    })
}

pub fn undeploy_mod(app_data: &Path, game_id: &str, mod_id: &str) -> AppResult<usize> {
    undeploy_mod_targets(app_data, game_id, mod_id)
}

fn run_post_deploy_hooks(
    profile: &GameProfile,
    game_root: &Path,
    staging: &Path,
    request: &DeployGameRequest,
) -> AppResult<()> {
    if profile.id == "cyberpunk2077" {
        let slugs = redmod_slugs_ordered(request, staging, profile);
        let slug_set: HashSet<String> = slugs.iter().cloned().collect();
        remove_orphan_redmod_folders(game_root, &slug_set)?;
        crate::cyberpunk::deploy_redmod(game_root, &slugs)?;
    }

    if profile.id == "baldursgate3" {
        let pak_names = bg3_pak_names(request, staging, profile);
        crate::bg3::sync_modsettings(&pak_names)?;
    }

    if profile.supports_plugins {
        crate::bethesda::sync_plugins_for_deploy(
            game_root,
            profile,
            request,
            staging,
        )?;
    }

    Ok(())
}

fn redmod_slugs_ordered(
    request: &DeployGameRequest,
    staging: &Path,
    profile: &GameProfile,
) -> Vec<String> {
    let mod_map: HashMap<_, _> = request.mods.iter().map(|m| (m.id.clone(), m)).collect();
    let mut slugs = Vec::new();
    for mod_id in ordered_enabled_ids(&request.mods, &request.enabled_ids) {
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
            request.deploy_path_override.as_deref(),
        );
        if normalized.mod_type == "cp77_redmod" && !slugs.contains(&slug) {
            slugs.push(slug);
        }
    }
    slugs
}

fn bg3_pak_names(
    request: &DeployGameRequest,
    staging: &Path,
    profile: &GameProfile,
) -> Vec<String> {
    let mod_map: HashMap<_, _> = request.mods.iter().map(|m| (m.id.clone(), m)).collect();
    let mut names = Vec::new();
    for mod_id in ordered_enabled_ids(&request.mods, &request.enabled_ids) {
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
            request.deploy_path_override.as_deref(),
        );
        if normalized.mod_type == "bg3_pak" {
            names.push(slug);
        }
    }
    names
}

pub fn run_preflight(
    game_dir: &str,
    profile_id: Option<&str>,
    mods: &[ModManifest],
    enabled_ids: &[String],
    staging_dir: &str,
    conflict_resolutions: &HashMap<String, String>,
    deploy_path_override: Option<&str>,
) -> crate::diagnostics::DiagnosticReport {
    use crate::diagnostics::{analyze_with_conflicts, file_conflict_issue};

    let profile = profile_id
        .and_then(profile_by_id)
        .unwrap_or(generic_profile());

    let staging = PathBuf::from(staging_dir);
    let game_root = PathBuf::from(game_dir);

    let mut issues: Vec<_> = analyze_with_conflicts(mods, enabled_ids)
        .issues
        .into_iter()
        .filter(|i| !i.id.starts_with("conflict-"))
        .collect();

    let mod_map: HashMap<_, _> = mods.iter().map(|m| (m.id.clone(), m)).collect();
    let mut file_owners: HashMap<String, Vec<(String, String)>> = HashMap::new();
    let mut normalized_paths = Vec::new();

    for mod_id in ordered_enabled_ids(mods, enabled_ids) {
        let Some(m) = mod_map.get(&mod_id) else { continue };
        if m.files.is_empty() {
            continue;
        }
        let slug = mod_slug_from_files(&m.files);
        let normalized = normalize_mod(
            &staging,
            &mod_id,
            &slug,
            &m.files,
            profile,
            deploy_path_override,
        );
        for file in normalized.files {
            let key = file.deploy_key();
            normalized_paths.push(key.clone());
            file_owners
                .entry(key)
                .or_default()
                .push((mod_id.clone(), m.name.clone()));
        }
    }

    for (file, owners) in file_owners {
        if owners.len() > 1 && !conflict_resolutions.contains_key(&file) {
            issues.push(file_conflict_issue(&file, &owners));
        }
    }

    issues.extend(
        check_requirements(&game_root, profile)
            .into_iter()
            .filter(|i| !i.id.is_empty()),
    );

    if profile.supports_plugins {
        if let Some(warn) = crate::bethesda::bsa_loose_files_advisory(&game_root) {
            issues.push(warn);
        }
    }

    if profile.id == "cyberpunk2077" {
        if let Some(warn) = crate::cyberpunk::staging_location_advisory(&staging, &game_root) {
            issues.push(warn);
        }
    }

    if let Some(warn) = profile_mismatch_warnings(profile, &normalized_paths) {
        issues.push(warn);
    }

    let blocking: Vec<_> = issues
        .iter()
        .filter(|i| i.id.starts_with("conflict-") || i.id.starts_with("missing-dep-"))
        .collect();

    let ready = blocking.is_empty();
    let summary = if ready {
        if issues.is_empty() {
            "Everything looks good — you're ready to install.".into()
        } else {
            "Ready to install — review warnings below.".into()
        }
    } else if blocking.len() == 1 {
        "One thing needs your decision before installing.".into()
    } else {
        format!(
            "{} items need your attention before installing.",
            blocking.len()
        )
    };

    crate::diagnostics::DiagnosticReport {
        ready,
        issues,
        summary,
    }
}

fn resolve_profile_for_request(request: &DeployGameRequest) -> &'static GameProfile {
    request
        .profile_id
        .as_deref()
        .and_then(profile_by_id)
        .unwrap_or(generic_profile())
}

fn mod_slug_from_files(files: &[String]) -> String {
    files
        .first()
        .and_then(|f| f.split('/').next())
        .unwrap_or("mod")
        .to_string()
}

fn build_summary(report: &DeployReport, profile: &GameProfile, count: usize) -> String {
    let path = &profile.default_mod_type().rel_path;
    if report.verified {
        format!("Verified: {count} file(s) linked into {path}.")
    } else {
        format!(
            "Partial: {} linked, {} missing, {} mismatched into {path}.",
            report.linked, report.missing, report.mismatched
        )
    }
}
