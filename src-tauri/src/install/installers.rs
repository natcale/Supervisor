/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
//! Structure-only installer chain run on ingest before file listing.

use crate::errors::{AppError, AppResult};
use crate::install::fomod;
use std::fs;
use std::path::{Path, PathBuf};

pub fn apply_install_chain(mod_root: &Path) -> AppResult<bool> {
    unwrap_single_top_folder(mod_root)?;
    Ok(fomod::has_fomod(mod_root))
}

fn unwrap_single_top_folder(mod_root: &Path) -> AppResult<()> {
    let entries: Vec<_> = fs::read_dir(mod_root)
        .map_err(AppError::Io)?
        .filter_map(|e| e.ok())
        .collect();

    if entries.len() != 1 {
        return Ok(());
    }

    let only = &entries[0];
    if !only.file_type().map_err(AppError::Io)?.is_dir() {
        return Ok(());
    }

    let inner = only.path();
    let inner_name = inner
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Don't unwrap known meaningful top-level folders.
    const KEEP: &[&str] = &[
        "data",
        "bepinex",
        "fomod",
        "qmods",
        "mods",
        "marvelgame",
    ];
    if KEEP.iter().any(|k| inner_name == *k) {
        return Ok(());
    }

    hoist_directory(&inner, mod_root)?;
    let _ = fs::remove_dir_all(&inner);
    Ok(())
}

pub(crate) fn hoist_directory(from: &Path, to: &Path) -> AppResult<()> {
    for entry in fs::read_dir(from).map_err(AppError::Io)? {
        let entry = entry.map_err(AppError::Io)?;
        let dest = to.join(entry.file_name());
        if dest.exists() {
            if dest.is_dir() {
                merge_dir(&entry.path(), &dest)?;
                fs::remove_dir_all(&entry.path()).map_err(AppError::Io)?;
            } else {
                fs::remove_file(&dest).map_err(AppError::Io)?;
                fs::rename(entry.path(), dest).map_err(AppError::Io)?;
            }
        } else {
            fs::rename(entry.path(), dest).map_err(AppError::Io)?;
        }
    }
    Ok(())
}

fn merge_dir(from: &Path, to: &Path) -> AppResult<()> {
    for entry in fs::read_dir(from).map_err(AppError::Io)? {
        let entry = entry.map_err(AppError::Io)?;
        let dest = to.join(entry.file_name());
        if entry.file_type().map_err(AppError::Io)?.is_dir() {
            fs::create_dir_all(&dest).map_err(AppError::Io)?;
            merge_dir(&entry.path(), &dest)?;
        } else if !dest.exists() {
            fs::rename(entry.path(), dest).map_err(AppError::Io)?;
        }
    }
    Ok(())
}

pub fn find_module_config(mod_root: &Path) -> Option<PathBuf> {
    for candidate in [
        mod_root.join("fomod").join("ModuleConfig.xml"),
        mod_root.join("ModuleConfig.xml"),
        mod_root.join("fomod").join("moduleconfig.xml"),
    ] {
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}
