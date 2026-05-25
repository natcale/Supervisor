/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::commands::app_data;
use crate::errors::UserFacingIssue;
use crate::library::{self, NexusMeta};
use crate::nexus::{fetch_latest_file_version, fetch_mod_details, validate_api_key, ModUpdateCheck, NexusModDetails};
use crate::secrets;

fn require_api_key() -> Result<String, UserFacingIssue> {
    secrets::get_nexus_api_key()
        .map_err(|e| e.to_user_issue())?
        .filter(|k| !k.trim().is_empty())
        .ok_or_else(|| {
            crate::errors::AppError::user(
                "Add a Nexus API key in Settings to use Nexus features. Get one at nexusmods.com → My Account → API.",
            )
            .to_user_issue()
        })
}

#[tauri::command]
pub async fn fetch_nexus_mod_metadata(
    _app: tauri::AppHandle,
    domain: String,
    mod_id: u64,
) -> Result<NexusModDetails, UserFacingIssue> {
    let api_key = require_api_key()?;
    fetch_mod_details(&domain, mod_id, &api_key)
        .await
        .map_err(|e| e.to_user_issue())
}

#[tauri::command]
pub async fn check_mod_updates(
    app: tauri::AppHandle,
    game_id: String,
) -> Result<Vec<ModUpdateCheck>, UserFacingIssue> {
    let data = app_data(&app)?;
    let api_key = require_api_key()?;

    let mut library = library::get_library(&data, &game_id).map_err(|e| e.to_user_issue())?;
    let mut results = Vec::new();
    let mut patches: Vec<(String, bool, Option<String>)> = Vec::new();

    let targets: Vec<_> = library
        .mods
        .iter()
        .filter_map(|m| {
            m.nexus.as_ref().map(|n| {
                (
                    m.id.clone(),
                    m.name.clone(),
                    n.clone(),
                )
            })
        })
        .collect();

    for (mod_id, mod_name, nexus) in targets {
        let latest = fetch_latest_file_version(
            &nexus.domain,
            nexus.mod_id as u64,
            nexus.file_id as u64,
            &api_key,
        )
        .await
        .map_err(|e| e.to_user_issue())?;

        let current = nexus.version.clone();
        let update_available = match (&current, &latest) {
            (Some(c), Some(l)) => l != c,
            (_, Some(_)) => true,
            _ => false,
        };

        patches.push((mod_id.clone(), update_available, latest.clone()));

        if update_available {
            results.push(ModUpdateCheck {
                mod_id,
                mod_name,
                current_version: current,
                latest_version: latest,
                update_available: true,
                nexus_mod_id: nexus.mod_id as u64,
                domain: nexus.domain,
            });
        }
    }

    for (mod_id, update_available, latest) in patches {
        if let Some(entry) = library.mods.iter_mut().find(|m| m.id == mod_id) {
            if let Some(meta) = &mut entry.nexus {
                meta.update_available = update_available;
                meta.latest_version = latest;
            }
        }
    }

    library.updated_at = library::now_ts();
    library::save_library(&data, &library).map_err(|e| e.to_user_issue())?;

    Ok(results)
}

#[tauri::command]
pub async fn enrich_mod_metadata(
    app: tauri::AppHandle,
    game_id: String,
    mod_id: String,
) -> Result<NexusMeta, UserFacingIssue> {
    let data = app_data(&app)?;
    let api_key = require_api_key()?;

    let mut library = library::get_library(&data, &game_id).map_err(|e| e.to_user_issue())?;
    let lib_mod = library
        .mods
        .iter()
        .find(|m| m.id == mod_id)
        .ok_or_else(|| crate::errors::AppError::user("Mod not found.").to_user_issue())?
        .clone();

    let nexus = lib_mod.nexus.clone().ok_or_else(|| {
        crate::errors::AppError::user("This mod has no Nexus metadata.").to_user_issue()
    })?;

    let details = fetch_mod_details(&nexus.domain, nexus.mod_id as u64, &api_key)
        .await
        .map_err(|e| e.to_user_issue())?;

    let updated = NexusMeta {
        mod_id: nexus.mod_id,
        file_id: nexus.file_id,
        domain: nexus.domain,
        version: details.version.or(nexus.version),
        author: details.author.or(nexus.author),
        picture_url: details.picture_url.or(nexus.picture_url),
        category: details.category.or(nexus.category),
        endorsed: nexus.endorsed,
        tracked: nexus.tracked,
        update_available: nexus.update_available,
        latest_version: nexus.latest_version,
        summary: details.summary.or(nexus.summary),
    };

    if let Some(entry) = library.mods.iter_mut().find(|m| m.id == mod_id) {
        entry.nexus = Some(updated.clone());
        entry.name = details.name;
    }
    library.updated_at = library::now_ts();
    library::save_library(&data, &library).map_err(|e| e.to_user_issue())?;

    Ok(updated)
}

#[tauri::command]
pub async fn validate_nexus_api_key(_app: tauri::AppHandle) -> Result<(), UserFacingIssue> {
    let api_key = require_api_key()?;
    validate_api_key(&api_key)
        .await
        .map_err(|e| e.to_user_issue())
}
