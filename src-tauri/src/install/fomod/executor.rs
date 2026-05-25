/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use super::parser::{folder_mappings_from_blocks, selected_option_blocks};
use crate::errors::{AppError, AppResult};
use crate::install::installers::{find_module_config, hoist_directory};
use std::fs;
use std::path::Path;

pub fn apply_fomod_selection(mod_root: &Path, selections: &[String]) -> AppResult<bool> {
    let Some(config_path) = find_module_config(mod_root) else {
        return Ok(false);
    };

    let xml = fs::read_to_string(&config_path).map_err(AppError::Io)?;
    let blocks = selected_option_blocks(&xml, selections);
    let mappings = folder_mappings_from_blocks(&blocks);

    if mappings.is_empty() {
        let option_blocks = extract_legacy_options(&xml);
        let mappings = folder_mappings_from_blocks(&option_blocks);
        if mappings.is_empty() {
            return Ok(false);
        }
        return apply_mappings(mod_root, &mappings);
    }

    apply_mappings(mod_root, &mappings)
}

fn extract_legacy_options(xml: &str) -> Vec<String> {
    let open = "<configOption";
    let close = "</configOption>";
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find(open) {
        let after = &rest[start..];
        if let Some(end) = after.find(close) {
            let block = &after[..end + close.len()];
            out.push(block.to_string());
            rest = &after[end + close.len()..];
        } else {
            break;
        }
    }
    if out.is_empty() {
        out.push(xml.to_string());
    }
    out.into_iter().take(1).collect()
}

fn apply_mappings(mod_root: &Path, mappings: &[(String, String)]) -> AppResult<bool> {
    let staging = mod_root.join(".fomod-staging");
    if staging.exists() {
        fs::remove_dir_all(&staging).map_err(AppError::Io)?;
    }
    fs::create_dir_all(&staging).map_err(AppError::Io)?;

    for (source, dest) in mappings {
        let from = mod_root.join(source);
        if !from.exists() {
            continue;
        }
        let to = staging.join(dest);
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).map_err(AppError::Io)?;
        }
        if from.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            if to.exists() {
                fs::remove_file(&to).map_err(AppError::Io)?;
            }
            fs::copy(&from, &to).map_err(AppError::Io)?;
        }
    }

    for entry in fs::read_dir(mod_root).map_err(AppError::Io)? {
        let entry = entry.map_err(AppError::Io)?;
        let path = entry.path();
        if path.file_name().and_then(|n| n.to_str()) == Some(".fomod-staging") {
            continue;
        }
        if path.is_dir() {
            fs::remove_dir_all(&path).map_err(AppError::Io)?;
        } else {
            fs::remove_file(&path).map_err(AppError::Io)?;
        }
    }

    hoist_directory(&staging, mod_root)?;
    fs::remove_dir_all(&staging).map_err(AppError::Io)?;

    let fomod_dir = mod_root.join("fomod");
    if fomod_dir.is_dir() {
        let _ = fs::remove_dir_all(&fomod_dir);
    }

    Ok(true)
}

fn copy_dir_recursive(from: &Path, to: &Path) -> AppResult<()> {
    fs::create_dir_all(to).map_err(AppError::Io)?;
    for entry in fs::read_dir(from).map_err(AppError::Io)? {
        let entry = entry.map_err(AppError::Io)?;
        let dest = to.join(entry.file_name());
        if entry.file_type().map_err(AppError::Io)?.is_dir() {
            copy_dir_recursive(&entry.path(), &dest)?;
        } else {
            fs::copy(entry.path(), dest).map_err(AppError::Io)?;
        }
    }
    Ok(())
}
