/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::errors::{AppError, AppResult, UserChoice, UserFacingIssue};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn find_redmod_exe(game_root: &Path) -> Option<PathBuf> {
    [
        game_root.join("tools/redmod/bin/redmod.exe"),
        game_root.join("tools/redmod/bin/x64/redmod.exe"),
        game_root.join("redmod.exe"),
    ]
    .into_iter()
    .find(|p| p.is_file())
}

pub fn deploy_redmod(game_root: &Path, slugs: &[String]) -> AppResult<()> {
    if slugs.is_empty() {
        return Ok(());
    }
    let Some(exe) = find_redmod_exe(game_root) else {
        return Err(AppError::user(
            "REDmod executable not found. Install/update Cyberpunk 2077 or verify game files.",
        ));
    };

    let mod_arg = slugs.join(",");
    let root = game_root.to_string_lossy();
    let output = Command::new(&exe)
        .args(["deploy", &format!("-root={root}"), &format!("-mod={mod_arg}")])
        .output()
        .map_err(AppError::Io)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(AppError::user(format!(
            "REDmod deploy failed: {stderr}{stdout}"
        )));
    }
    Ok(())
}

pub fn staging_location_advisory(staging: &Path, game_root: &Path) -> Option<UserFacingIssue> {
    let Ok(staging_canon) = staging.canonicalize() else {
        return None;
    };
    let Ok(game_canon) = game_root.canonicalize() else {
        return None;
    };
    if !staging_canon.starts_with(&game_canon) {
        return None;
    }
    Some(UserFacingIssue {
        id: "cp77-staging-in-game".into(),
        title: "Staging folder is inside the game install".into(),
        explanation:
            "Cyberpunk modding works best when your Supervisor staging folder is separate from the game directory."
                .into(),
        impact: "REDmod and archive mods may conflict or fail to deploy correctly.".into(),
        choices: vec![UserChoice {
            id: "open-settings".into(),
            label: "Change staging folder".into(),
            description: "Pick a folder outside the game install.".into(),
            recommended: true,
        }],
    })
}

pub fn cyberpunk_launch_args(redmod_enabled: bool) -> Vec<String> {
    if redmod_enabled {
        vec!["-modded".into()]
    } else {
        Vec::new()
    }
}

pub fn cyberpunk_exe(game_root: &Path) -> Option<PathBuf> {
    let exe = game_root.join("bin/x64/Cyberpunk2077.exe");
    if exe.is_file() {
        return Some(exe);
    }
    let alt = game_root.join("Cyberpunk2077.exe");
    if alt.is_file() {
        return Some(alt);
    }
    None
}
