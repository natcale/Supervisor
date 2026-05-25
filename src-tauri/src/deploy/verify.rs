/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
#![allow(dead_code)]
use crate::deploy::manifest::{DeployManifest, ManifestTarget};
use crate::errors::{UserChoice, UserFacingIssue};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployReport {
    pub verified: bool,
    pub linked: usize,
    pub missing: usize,
    pub mismatched: usize,
    pub issues: Vec<UserFacingIssue>,
    pub profile_warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileVerifyResult {
    pub rel_path: String,
    pub ok: bool,
    pub reason: Option<String>,
}

pub fn verify_manifest(manifest: &DeployManifest) -> DeployReport {
    let mut linked = 0usize;
    let mut missing = 0usize;
    let mut mismatched = 0usize;
    let mut issues = Vec::new();

    for target in &manifest.targets {
        let deploy_path = Path::new(&target.deploy_root).join(&target.rel_path);
        let source_path = Path::new(&target.source);

        if !deploy_path.is_file() {
            missing += 1;
            issues.push(missing_issue(&target.rel_path, &deploy_path));
            continue;
        }

        if !source_path.is_file() {
            mismatched += 1;
            issues.push(mismatch_issue(
                &target.rel_path,
                "Staging source file is missing — the mod may have been moved.",
            ));
            continue;
        }

        let same_inode = crate::hardlink::same_file(&deploy_path, source_path);

        if same_inode {
            linked += 1;
        } else {
            mismatched += 1;
            issues.push(mismatch_issue(
                &target.rel_path,
                "Deployed file exists but does not match the staging copy.",
            ));
        }
    }

    let verified = missing == 0 && mismatched == 0;

    DeployReport {
        verified,
        linked,
        missing,
        mismatched,
        issues,
        profile_warning: None,
    }
}

pub fn verify_target(target: &ManifestTarget) -> FileVerifyResult {
    let deploy_path = Path::new(&target.deploy_root).join(&target.rel_path);
    let source_path = Path::new(&target.source);

    if !deploy_path.is_file() {
        return FileVerifyResult {
            rel_path: target.rel_path.clone(),
            ok: false,
            reason: Some("File is missing from the game folder.".into()),
        };
    }

    if !source_path.is_file() {
        return FileVerifyResult {
            rel_path: target.rel_path.clone(),
            ok: false,
            reason: Some("Staging source is missing.".into()),
        };
    }

    let ok = crate::hardlink::same_file(&deploy_path, source_path);

    FileVerifyResult {
        rel_path: target.rel_path.clone(),
        ok,
        reason: if ok {
            None
        } else {
            Some("Deployed file does not match staging.".into())
        },
    }
}

fn missing_issue(rel: &str, path: &Path) -> UserFacingIssue {
    UserFacingIssue {
        id: format!("missing-{}", rel.replace('/', "-")),
        title: format!("\"{}\" was not found in your game folder", file_name(rel)),
        explanation: format!(
            "Supervisor expected this file at \"{}\" but it is missing.",
            path.display()
        ),
        impact: "The mod may not work until you reinstall.".into(),
        choices: vec![UserChoice {
            id: "reinstall".into(),
            label: "Reinstall mods".into(),
            description: "Run install again to restore missing files.".into(),
            recommended: true,
        }],
    }
}

fn mismatch_issue(rel: &str, detail: &str) -> UserFacingIssue {
    UserFacingIssue {
        id: format!("mismatch-{}", rel.replace('/', "-")),
        title: format!("\"{}\" does not match what was installed", file_name(rel)),
        explanation: detail.into(),
        impact: "Your game may be using an outdated or modified copy of this file.".into(),
        choices: vec![UserChoice {
            id: "reinstall".into(),
            label: "Reinstall mods".into(),
            description: "Replace the file with a fresh link from staging.".into(),
            recommended: true,
        }],
    }
}

fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}
