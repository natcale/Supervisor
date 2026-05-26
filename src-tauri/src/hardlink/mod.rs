/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
//! Low-level hardlink engine for same-partition file linking.
#![allow(dead_code)]
use crate::errors::{AppError, AppResult, UserChoice, UserFacingIssue};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentRequest {
    pub staging_dir: String,
    pub game_dir: String,
    pub data_dir: Option<String>,
    pub root_files: Vec<RootFileEntry>,
    pub mod_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootFileEntry {
    pub source: String,
    pub target_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentResult {
    pub deployed_files: usize,
    pub root_files: usize,
    pub mod_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartitionCheckResult {
    pub same_partition: bool,
    pub staging_volume: String,
    pub game_volume: String,
    pub guidance: Option<UserFacingIssue>,
}

pub fn check_same_partition(staging: &Path, game: &Path) -> AppResult<PartitionCheckResult> {
    let staging_vol = volume_id(staging)?;
    let game_vol = volume_id(game)?;
    let same = staging_vol == game_vol;

    let guidance = if same {
        None
    } else {
        Some(UserFacingIssue {
            id: "partition-mismatch".into(),
            title: "Your mod folder and game are on different drives".into(),
            explanation: format!(
                "Supervisor installs mods using space-efficient links that only work when both folders live on the same drive. Your staging folder is on \"{staging_vol}\" but the game is on \"{game_vol}\"."
            ),
            impact: "Installing now would fail or copy files inefficiently. Move your staging folder to the same drive as the game to continue.".into(),
            choices: vec![
                UserChoice {
                    id: "open-settings".into(),
                    label: "Change staging folder".into(),
                    description: "Pick a folder on the same drive as your game.".into(),
                    recommended: true,
                },
                UserChoice {
                    id: "cancel".into(),
                    label: "Cancel for now".into(),
                    description: "Return without making changes.".into(),
                    recommended: false,
                },
            ],
        })
    };

    Ok(PartitionCheckResult {
        same_partition: same,
        staging_volume: staging_vol,
        game_volume: game_vol,
        guidance,
    })
}

pub fn deploy_hardlinks(request: &DeploymentRequest) -> AppResult<DeploymentResult> {
    let staging = PathBuf::from(&request.staging_dir);
    let game = PathBuf::from(&request.game_dir);

    let partition = check_same_partition(&staging, &game)?;
    if !partition.same_partition {
        return Err(AppError::user(
            partition
                .guidance
                .map(|g| g.explanation)
                .unwrap_or_else(|| "Staging and game folders must be on the same drive.".into()),
        ));
    }

    let data_dir = resolve_data_dir(&game, request.data_dir.as_ref().map(PathBuf::from))?;
    fs::create_dir_all(&data_dir).map_err(AppError::Io)?;

    let mut root_count = 0;
    for entry in &request.root_files {
        let source = staging.join(&entry.source);
        let target = game.join(&entry.target_name);
        hardlink_file(&source, &target)?;
        root_count += 1;
    }

    let mut mod_count = 0;
    for rel in &request.mod_files {
        let source = staging.join(rel);
        let target = data_dir.join(rel);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(AppError::Io)?;
        }
        hardlink_file(&source, &target)?;
        mod_count += 1;
    }

    Ok(DeploymentResult {
        deployed_files: root_count + mod_count,
        root_files: root_count,
        mod_files: mod_count,
    })
}

pub fn same_file(a: &Path, b: &Path) -> bool {
    if !a.exists() || !b.exists() {
        return false;
    }
    #[cfg(windows)]
    {
        return file_identity(a) == file_identity(b);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let Ok(ma) = fs::metadata(a) else {
            return false;
        };
        let Ok(mb) = fs::metadata(b) else {
            return false;
        };
        ma.dev() == mb.dev() && ma.ino() == mb.ino()
    }
    #[cfg(not(any(windows, unix)))]
    {
        let _ = (a, b);
        false
    }
}

#[cfg(windows)]
fn file_identity(path: &Path) -> Option<(u32, u64)> {
    use std::fs::OpenOptions;
    use std::os::windows::fs::OpenOptionsExt;
    use std::os::windows::io::AsRawHandle;

    #[repr(C)]
    struct ByHandleFileInformation {
        dw_file_attributes: u32,
        ft_creation_time: u64,
        ft_last_access_time: u64,
        ft_last_write_time: u64,
        dw_volume_serial_number: u32,
        n_file_size_high: u32,
        n_file_size_low: u32,
        n_number_of_links: u32,
        n_file_index_high: u32,
        n_file_index_low: u32,
    }

    extern "system" {
        fn GetFileInformationByHandle(
            h_file: *mut std::ffi::c_void,
            lp_file_information: *mut ByHandleFileInformation,
        ) -> i32;
    }

    const FILE_SHARE_READ: u32 = 0x00000001;
    const FILE_SHARE_WRITE: u32 = 0x00000002;
    const FILE_SHARE_DELETE: u32 = 0x00000004;

    let file = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .open(path)
        .ok()?;
    let handle = file.as_raw_handle();
    let mut info = ByHandleFileInformation {
        dw_file_attributes: 0,
        ft_creation_time: 0,
        ft_last_access_time: 0,
        ft_last_write_time: 0,
        dw_volume_serial_number: 0,
        n_file_size_high: 0,
        n_file_size_low: 0,
        n_number_of_links: 0,
        n_file_index_high: 0,
        n_file_index_low: 0,
    };
    let ok = unsafe { GetFileInformationByHandle(handle as _, &mut info) };
    if ok == 0 {
        return None;
    }
    let index = (info.n_file_index_high as u64) << 32 | info.n_file_index_low as u64;
    Some((info.dw_volume_serial_number, index))
}

/// Remove a deployed hardlink only if it still points at the staging source we linked.
pub fn remove_managed_link(target: &Path, expected_source: &Path) -> AppResult<bool> {
    if !target.is_file() {
        return Ok(false);
    }
    if !same_file(target, expected_source) {
        return Ok(false);
    }
    fs::remove_file(target).map_err(AppError::Io)?;
    Ok(true)
}

pub fn hardlink_file(source: &Path, target: &Path) -> AppResult<()> {
    if !source.is_file() {
        return Err(AppError::user(format!(
            "Expected a mod file at \"{}\" but it wasn't found.",
            source.display()
        )));
    }
    if target.exists() {
        fs::remove_file(target).map_err(AppError::Io)?;
    }
    fs::hard_link(source, target).map_err(|e| {
        AppError::user(format!(
            "Could not link \"{}\" into your game folder: {e}. Both folders must be on the same drive.",
            source.file_name().and_then(|n| n.to_str()).unwrap_or("file")
        ))
    })
}

fn resolve_data_dir(game: &Path, explicit: Option<PathBuf>) -> AppResult<PathBuf> {
    if let Some(path) = explicit.filter(|p| !p.as_os_str().is_empty()) {
        return Ok(path);
    }

    if let Some(found) = infer_data_dir(game) {
        return Ok(found);
    }

    Ok(game.join("Data"))
}

fn infer_data_dir(game: &Path) -> Option<PathBuf> {
    for sub in ["Data", "data", "Mods", "mods"] {
        let candidate = game.join(sub);
        if candidate.is_dir() {
            return Some(candidate);
        }
    }
    None
}

fn volume_id(path: &Path) -> AppResult<String> {
    #[cfg(windows)]
    {
        let canonical = path.canonicalize().map_err(AppError::Io)?;
        let root = canonical
            .components()
            .next()
            .and_then(|c| c.as_os_str().to_str())
            .ok_or_else(|| AppError::user("Could not determine drive for path."))?;

        let root_path = format!("{root}\\");
        let wide: Vec<u16> = root_path.encode_utf16().chain(std::iter::once(0)).collect();

        let mut serial: u32 = 0;
        let ok = unsafe {
            GetVolumeInformationW(
                wide.as_ptr(),
                std::ptr::null_mut(),
                0,
                &mut serial,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
            )
        };

        if ok != 0 {
            return Ok(format!("vol-{serial:08X}"));
        }
        return Ok(root.to_string());
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let meta = fs::metadata(path).map_err(AppError::Io)?;
        Ok(format!("dev-{}", meta.dev()))
    }
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetVolumeInformationW(
        lp_root_path_name: *const u16,
        lp_volume_name_buffer: *mut u16,
        n_volume_name_size: u32,
        lp_volume_serial_number: *mut u32,
        lp_maximum_component_length: *mut u32,
        lp_file_system_flags: *mut u32,
        lp_file_system_name_buffer: *mut u16,
        n_file_system_name_size: u32,
    ) -> i32;
}
