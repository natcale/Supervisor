/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use super::catalog::engine_key;
use super::fixtures::{per_mod_folder_name, seed_game_tree, write_staging_mod};
use crate::deploy::manifest::{DeployManifest, manifest_path, read_manifest};
use crate::deploy::{deploy_game, purge_deployment, undeploy_mod, DeployGameRequest};
use crate::games::mod_path_for_type;
use crate::diagnostics::ModManifest;
use crate::errors::AppResult;
use crate::games::{profile_by_id, GameProfile, MergeMode};
use crate::hardlink::{check_same_partition, same_file};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FlowError {
    pub profile_id: String,
    pub step: String,
    pub detail: String,
}

#[derive(Debug)]
pub struct FlowReport {
    pub passed: usize,
    pub failures: Vec<FlowError>,
}

impl fmt::Display for FlowReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "deploy flow: {} passed, {} failed",
            self.passed,
            self.failures.len()
        )?;
        for err in &self.failures {
            writeln!(f, "  [{}] {} — {}", err.profile_id, err.step, err.detail)?;
        }
        Ok(())
    }
}

pub struct Sandbox {
    pub root: PathBuf,
    pub game_root: PathBuf,
    pub staging: PathBuf,
    pub app_data: PathBuf,
}

impl Sandbox {
    pub fn new() -> AppResult<Self> {
        let root = std::env::temp_dir().join(format!(
            "supervisor-deploy-flow-{}",
            uuid::Uuid::new_v4()
        ));
        let game_root = root.join("game");
        let staging = root.join("staging");
        let app_data = root.join("app_data");
        fs::create_dir_all(&game_root).map_err(crate::errors::AppError::Io)?;
        fs::create_dir_all(&staging).map_err(crate::errors::AppError::Io)?;
        fs::create_dir_all(&app_data).map_err(crate::errors::AppError::Io)?;

        let partition = check_same_partition(&staging, &game_root)?;
        if !partition.same_partition {
            return Err(crate::errors::AppError::user(
                "Deploy flow tests require staging and game sandbox on the same drive.",
            ));
        }

        Ok(Self {
            root,
            game_root,
            staging,
            app_data,
        })
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

pub fn run_profile_flow(profile_id: &str) -> Result<(), FlowError> {
    let profile = profile_by_id(profile_id).ok_or_else(|| FlowError {
        profile_id: profile_id.into(),
        step: "lookup".into(),
        detail: "unknown profile id".into(),
    })?;

    let sandbox = Sandbox::new().map_err(|e| FlowError {
        profile_id: profile_id.into(),
        step: "sandbox".into(),
        detail: e.to_string(),
    })?;

    run_flow_in_sandbox(profile, &sandbox).map_err(|e| FlowError {
        profile_id: profile_id.into(),
        step: e.step,
        detail: e.detail,
    })
}

struct StepError {
    step: String,
    detail: String,
}

fn run_flow_in_sandbox(profile: &GameProfile, sandbox: &Sandbox) -> Result<(), StepError> {
    let engine = engine_key(profile);

    seed_game_tree(&sandbox.game_root, profile, engine).map_err(|e| StepError {
        step: "seed_game".into(),
        detail: e.to_string(),
    })?;

    let fixture = write_staging_mod(&sandbox.staging, profile, engine).map_err(|e| StepError {
        step: "write_staging".into(),
        detail: e.to_string(),
    })?;

    let mod_folder = per_mod_folder_name(profile, &sandbox.staging, &fixture);

    let mods = vec![ModManifest {
        id: fixture.mod_id.clone(),
        name: "Flow Test Mod".into(),
        files: fixture.files.clone(),
        dependencies: vec![],
    }];

    let request = DeployGameRequest {
        game_id: profile.id.clone(),
        game_dir: sandbox.game_root.to_string_lossy().into_owned(),
        profile_id: Some(profile.id.clone()),
        staging_dir: sandbox.staging.to_string_lossy().into_owned(),
        mods: mods.clone(),
        enabled_ids: vec![fixture.mod_id.clone()],
        conflict_resolutions: HashMap::new(),
        ignore_requirements: false,
        deploy_path_override: None,
    };

    let deploy = deploy_game(&sandbox.app_data, &request, true).map_err(|e| StepError {
        step: "deploy".into(),
        detail: e.to_string(),
    })?;

    if !deploy.report.verified {
        return Err(StepError {
            step: "verify_report".into(),
            detail: format!(
                "linked={} missing={} mismatched={} issues={}",
                deploy.report.linked,
                deploy.report.missing,
                deploy.report.mismatched,
                deploy.report.issues.len()
            ),
        });
    }

    if deploy.manifest.targets.is_empty() {
        return Err(StepError {
            step: "deploy".into(),
            detail: "manifest has no targets".into(),
        });
    }

    assert_manifest_targets(
        &deploy.manifest,
        profile,
        &sandbox.game_root,
        mod_folder.as_deref(),
    )
    .map_err(|e| StepError {
        step: "assert_deployed".into(),
        detail: e,
    })?;

    let removed = undeploy_mod(&sandbox.app_data, &profile.id, &fixture.mod_id).map_err(|e| {
        StepError {
            step: "undeploy".into(),
            detail: e.to_string(),
        }
    })?;

    if removed == 0 {
        return Err(StepError {
            step: "undeploy".into(),
            detail: "removed 0 files".into(),
        });
    }

    assert_undeployed(&deploy.manifest, &sandbox.game_root, mod_folder.as_deref()).map_err(|e| {
        StepError {
            step: "assert_undeployed".into(),
            detail: e,
        }
    })?;

    purge_deployment(&sandbox.app_data, &profile.id).map_err(|e| StepError {
        step: "purge".into(),
        detail: e.to_string(),
    })?;

    if read_manifest(&manifest_path(&sandbox.app_data, &profile.id))
        .ok()
        .flatten()
        .is_some()
    {
        return Err(StepError {
            step: "purge".into(),
            detail: "manifest still present after purge".into(),
        });
    }

    Ok(())
}

fn assert_manifest_targets(
    manifest: &DeployManifest,
    profile: &GameProfile,
    game_root: &Path,
    mod_folder: Option<&str>,
) -> Result<(), String> {
    for target in &manifest.targets {
        let deploy_path = Path::new(&target.deploy_root).join(&target.rel_path);
        let source = Path::new(&target.source);

        if !deploy_path.is_file() {
            return Err(format!(
                "deploy file missing at {}",
                deploy_path.display()
            ));
        }
        if !source.is_file() {
            return Err(format!("staging source missing at {}", source.display()));
        }
        if !same_file(&deploy_path, source) {
            return Err(format!(
                "not a hardlink: {} -> {}",
                source.display(),
                deploy_path.display()
            ));
        }
    }

    if profile.merge_mode == MergeMode::PerModFolder {
        let folder = mod_folder.ok_or_else(|| "missing per-mod folder name".to_string())?;
        let deploy_root = mod_path_for_type(
            &game_root.to_path_buf(),
            profile.default_mod_type(),
            &profile.id,
        );
        let mod_dir = deploy_root.join(folder);
        if !mod_dir.is_dir() {
            return Err(format!(
                "per-mod folder not created at {}",
                mod_dir.display()
            ));
        }
        let has_content = fs::read_dir(&mod_dir)
            .map_err(|e| e.to_string())?
            .flatten()
            .next()
            .is_some();
        if !has_content {
            return Err(format!(
                "per-mod folder is empty at {}",
                mod_dir.display()
            ));
        }
    }

    Ok(())
}

fn assert_undeployed(
    manifest: &DeployManifest,
    game_root: &Path,
    mod_folder: Option<&str>,
) -> Result<(), String> {
    for target in &manifest.targets {
        let deploy_path = Path::new(&target.deploy_root).join(&target.rel_path);
        if deploy_path.exists() {
            return Err(format!(
                "file still present after undeploy: {}",
                deploy_path.display()
            ));
        }
    }

    if let Some(folder) = mod_folder {
        if let Some(first) = manifest.targets.first() {
            let deploy_root = Path::new(&first.deploy_root);
            let mod_dir = deploy_root.join(folder);
            if mod_dir.exists() {
                return Err(format!(
                    "per-mod folder still present after undeploy: {}",
                    mod_dir.display()
                ));
            }
        }
    }

    let _ = game_root;
    Ok(())
}
