/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::errors::{AppError, AppResult, UserChoice, UserFacingIssue};
use std::fs;
use std::path::Path;
use std::time::SystemTime;

const VANILLA_ARCHIVES: &[&str] = &[
    "Skyrim - Misc.bsa",
    "Skyrim - Shaders.bsa",
    "Skyrim - Textures.bsa",
    "Skyrim - Voices_en0.bsa",
    "Skyrim - Meshes.bsa",
    "Skyrim - Animations.bsa",
    "Skyrim - Interface.bsa",
    "Skyrim - Patch.bsa",
    "Skyrim - Sounds.bsa",
    "Update.bsa",
    "Dawnguard.bsa",
    "HearthFires.bsa",
    "Dragonborn.bsa",
    "Fallout4 - Meshes.ba2",
    "Fallout4 - Textures.ba2",
    "Fallout4 - Voices_en.ba2",
    "Fallout4 - Shaders.ba2",
    "Fallout4 - Interface.ba2",
    "Fallout4 - Materials.ba2",
    "Fallout4 - Misc.ba2",
    "Starfield - Meshes.ba2",
    "Starfield - Textures.ba2",
];

pub fn bsa_loose_files_advisory(game_root: &Path) -> Option<UserFacingIssue> {
    let data = game_root.join("Data");
    if !data.is_dir() {
        return None;
    }
    let has_loose = fs::read_dir(&data).ok()?.flatten().any(|e| {
        e.path().is_file()
            && e.path()
                .extension()
                .and_then(|x| x.to_str())
                .is_some_and(|ext| !matches!(ext.to_ascii_lowercase().as_str(), "bsa" | "ba2" | "esm" | "esp" | "esl"))
    });
    if !has_loose {
        return None;
    }
    if bsa_timestamps_ok(&data) {
        return None;
    }
    Some(UserFacingIssue {
        id: "bsa-loose-files".into(),
        title: "Loose files may not get loaded".into(),
        explanation: "Bethesda games may ignore loose files unless vanilla BSA/BA2 archives have newer timestamps.".into(),
        impact: "Texture and mesh mods may not appear in-game until archive timestamps are fixed.".into(),
        choices: vec![UserChoice {
            id: "fix-bsa-timestamps".into(),
            label: "Fix archive timestamps".into(),
            description: "Update vanilla BSA/BA2 file times (safe, recommended).".into(),
            recommended: true,
        }],
    })
}

pub fn fix_bsa_timestamps(game_root: &Path) -> AppResult<usize> {
    let data = game_root.join("Data");
    if !data.is_dir() {
        return Ok(0);
    }
    let now = filetime_now();
    let mut updated = 0usize;
    for entry in fs::read_dir(&data).map_err(AppError::Io)?.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if ext != "bsa" && ext != "ba2" {
            continue;
        }
        if !is_vanilla_archive(&name) {
            continue;
        }
        set_file_times(&path, now)?;
        updated += 1;
    }
    Ok(updated)
}

fn bsa_timestamps_ok(data: &Path) -> bool {
    let Ok(entries) = fs::read_dir(data) else {
        return true;
    };
    let mut newest_loose = SystemTime::UNIX_EPOCH;
    let mut oldest_bsa = SystemTime::UNIX_EPOCH;
    let mut found_bsa = false;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Ok(meta) = fs::metadata(&path) else {
            continue;
        };
        let Ok(modified) = meta.modified() else {
            continue;
        };
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if ext == "bsa" || ext == "ba2" {
            if is_vanilla_archive(&entry.file_name().to_string_lossy()) {
                if !found_bsa || modified < oldest_bsa {
                    oldest_bsa = modified;
                }
                found_bsa = true;
            }
        } else if !matches!(ext.as_str(), "esm" | "esp" | "esl") {
            if modified > newest_loose {
                newest_loose = modified;
            }
        }
    }
    if !found_bsa {
        return true;
    }
    oldest_bsa >= newest_loose
}

fn is_vanilla_archive(name: &str) -> bool {
    VANILLA_ARCHIVES
        .iter()
        .any(|v| name.eq_ignore_ascii_case(v))
        || name.starts_with("Skyrim")
        || name.starts_with("Fallout4")
        || name.starts_with("Starfield")
}

fn filetime_now() -> SystemTime {
    SystemTime::now()
}

#[cfg(windows)]
fn set_file_times(path: &Path, time: SystemTime) -> AppResult<()> {
    use std::os::windows::fs::OpenOptionsExt;
    const FILE_WRITE_ATTRIBUTES: u32 = 0x0100;
    let file = std::fs::OpenOptions::new()
        .write(true)
        .custom_flags(FILE_WRITE_ATTRIBUTES)
        .open(path)
        .map_err(AppError::Io)?;
    file.set_modified(time).map_err(AppError::Io)?;
    Ok(())
}

#[cfg(not(windows))]
fn set_file_times(_path: &Path, _time: SystemTime) -> AppResult<()> {
    Ok(())
}
