/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::errors::UserChoice;
use crate::errors::UserFacingIssue;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModManifest {
    pub id: String,
    pub name: String,
    pub files: Vec<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticReport {
    pub ready: bool,
    pub issues: Vec<UserFacingIssue>,
    pub summary: String,
}

pub fn analyze_with_conflicts(mods: &[ModManifest], enabled_ids: &[String]) -> DiagnosticReport {
    let enabled: HashSet<_> = enabled_ids.iter().cloned().collect();
    let mod_map: HashMap<_, _> = mods.iter().map(|m| (m.id.clone(), m)).collect();

    let mut issues = Vec::new();
    let mut file_owners: HashMap<String, Vec<(String, String)>> = HashMap::new();

    for mod_id in &enabled {
        let Some(m) = mod_map.get(mod_id) else {
            continue;
        };
        for dep in &m.dependencies {
            if !enabled.contains(dep) {
                issues.push(missing_dependency_issue(
                    &m.name,
                    dep,
                    mod_map.get(dep).copied(),
                ));
            }
        }
        for file in &m.files {
            file_owners
                .entry(normalize_path(file))
                .or_default()
                .push((mod_id.clone(), m.name.clone()));
        }
    }

    for (file, owners) in file_owners {
        if owners.len() > 1 {
            issues.push(file_conflict_issue(&file, &owners));
        }
    }

    let ready = issues.iter().all(|i| !is_blocking(i));
    let blocking_count = issues.iter().filter(|i| is_blocking(i)).count();
    let summary = if blocking_count == 0 {
        if issues.is_empty() {
            "Everything looks good — you're ready to install.".into()
        } else {
            "Ready to install — review warnings below.".into()
        }
    } else if blocking_count == 1 {
        "One thing needs your decision before installing.".into()
    } else {
        format!("{blocking_count} items need your attention before installing.")
    };

    DiagnosticReport {
        ready,
        issues,
        summary,
    }
}

fn is_blocking(issue: &UserFacingIssue) -> bool {
    issue.id.starts_with("conflict-") || issue.id.starts_with("missing-dep-")
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/").to_lowercase()
}

fn missing_dependency_issue(
    mod_name: &str,
    dep_id: &str,
    dep: Option<&ModManifest>,
) -> UserFacingIssue {
    let dep_name = dep.map(|d| d.name.as_str()).unwrap_or(dep_id);
    UserFacingIssue {
        id: format!("missing-dep-{dep_id}"),
        title: format!("\"{mod_name}\" needs another mod to work"),
        explanation: format!(
            "\"{mod_name}\" expects \"{dep_name}\" to be enabled. Without it, the mod may crash your game or behave unpredictably."
        ),
        impact: "You can enable the required mod, pick an alternative, or continue without it.".into(),
        choices: vec![
            UserChoice {
                id: format!("enable-{dep_id}"),
                label: format!("Enable {dep_name}"),
                description: "Add the required mod to your active list.".into(),
                recommended: true,
            },
            UserChoice {
                id: format!("skip-{dep_id}"),
                label: format!("Install {mod_name} anyway"),
                description: "Proceed knowing some features may not work.".into(),
                recommended: false,
            },
        ],
    }
}

pub fn file_conflict_issue(file: &str, owners: &[(String, String)]) -> UserFacingIssue {
    let file_display = file.rsplit('/').next().unwrap_or(file);
    let names: Vec<_> = owners.iter().map(|(_, name)| name.as_str()).collect();
    UserFacingIssue {
        id: format!("conflict-{file}"),
        title: format!("Two mods both change \"{file_display}\""),
        explanation: format!(
            "{} both include a file named \"{file_display}\". Only one version can be active at a time.",
            names.join(" and ")
        ),
        impact: "Choosing which mod wins determines which version of this file your game uses.".into(),
        choices: owners
            .iter()
            .map(|(mod_id, name)| UserChoice {
                id: format!("prefer-{mod_id}"),
                label: format!("Use the version from {name}"),
                description: format!("Keep {name}'s copy of \"{file_display}\"."),
                recommended: false,
            })
            .chain(std::iter::once(UserChoice {
                id: "cancel".into(),
                label: "Cancel installation".into(),
                description: "Go back and adjust your mod selection.".into(),
                recommended: false,
            }))
            .collect(),
    }
}
