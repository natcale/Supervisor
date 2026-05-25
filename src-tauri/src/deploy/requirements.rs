/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::errors::{UserChoice, UserFacingIssue};
use crate::games::{GameProfile, MergeMode};
use std::fs;
use std::path::Path;

pub fn check_requirements(game_root: &Path, profile: &GameProfile) -> Vec<UserFacingIssue> {
    let mut issues = Vec::new();

    for req in &profile.requirements {
        let target = game_root.join(&req.path);

        if req.create_if_missing {
            if !target.exists() {
                if fs::create_dir_all(&target).is_ok() {
                    issues.push(created_folder_issue(&req.label, &target));
                } else {
                    issues.push(missing_requirement_issue(req, &target, false));
                }
            }
            continue;
        }

        if !target.exists() {
            issues.push(missing_requirement_issue(req, &target, req.optional));
        }
    }

    if profile.merge_mode == MergeMode::PerModFolder {
        issues.push(smf_note_if_hitman(profile));
    }

    issues.into_iter().filter(|i| !i.id.is_empty()).collect()
}

fn missing_requirement_issue(
    req: &crate::games::RequirementDef,
    path: &Path,
    optional: bool,
) -> UserFacingIssue {
    UserFacingIssue {
        id: format!("req-{}", req.id),
        title: format!("{} is not installed", req.label),
        explanation: format!(
            "This game expects \"{}\" at \"{}\" before mods can work.",
            req.label,
            path.display()
        ),
        impact: if optional {
            "You can continue, but mods may not load correctly.".into()
        } else {
            "Installing now will place files in the right folder, but the game may ignore them until this tool is set up.".into()
        },
        choices: vec![
            UserChoice {
                id: "open-game-folder".into(),
                label: "Open game folder".into(),
                description: "Install the required tool, then try again.".into(),
                recommended: true,
            },
            UserChoice {
                id: "continue-anyway".into(),
                label: "Install mods anyway".into(),
                description: "Deploy files knowing the game may not load them yet.".into(),
                recommended: false,
            },
        ],
    }
}

fn created_folder_issue(label: &str, path: &Path) -> UserFacingIssue {
    UserFacingIssue {
        id: String::new(),
        title: format!("Created {label}"),
        explanation: format!("Supervisor created \"{}\" for mod hardlink.", path.display()),
        impact: "You can install .pak mods into this folder.".into(),
        choices: vec![],
    }
}

fn smf_note_if_hitman(profile: &GameProfile) -> UserFacingIssue {
    if profile.id != "hitman3" {
        return UserFacingIssue {
            id: String::new(),
            title: String::new(),
            explanation: String::new(),
            impact: String::new(),
            choices: vec![],
        };
    }

    UserFacingIssue {
        id: "smf-apply-note".into(),
        title: "SMF patch step is separate".into(),
        explanation:
            "Supervisor deploys mods into Simple Mod Framework/Mods/. After installing, run SMF's patch/apply step so the game loads your mods."
                .into(),
        impact: "Files will be linked correctly, but Hitman won't use them until SMF applies the patch.".into(),
        choices: vec![],
    }
}

pub fn profile_mismatch_warnings(
    profile: &GameProfile,
    normalized_paths: &[String],
) -> Option<UserFacingIssue> {
    if profile.id == "marvelrivals" {
        let bad: Vec<_> = normalized_paths
            .iter()
            .filter(|p| {
                let lower = p.to_lowercase();
                !lower.ends_with(".pak") && !lower.ends_with(".ucas") && !lower.ends_with(".utoc")
            })
            .take(3)
            .cloned()
            .collect();
        if !bad.is_empty() {
            return Some(UserFacingIssue {
                id: "profile-mismatch".into(),
                title: "Some files don't look like Marvel Rivals mods".into(),
                explanation: format!(
                    "Marvel Rivals expects .pak mod files. Found: {}.",
                    bad.join(", ")
                ),
                impact: "These files may be deployed to the wrong location or ignored by the game.".into(),
                choices: vec![UserChoice {
                    id: "continue-anyway".into(),
                    label: "Continue anyway".into(),
                    description: "Proceed with installation.".into(),
                    recommended: false,
                }],
            });
        }
    }

    if profile.id == "generic-data" {
        return Some(UserFacingIssue {
            id: "generic-profile".into(),
            title: "Using generic Data/ deploy target".into(),
            explanation:
                "Supervisor doesn't have a specific profile for this game, so mods will go into a Data/ folder."
                    .into(),
            impact: "Files may end up in the wrong place. Pick a deploy profile if you know where this game stores mods.".into(),
            choices: vec![],
        });
    }

    None
}
