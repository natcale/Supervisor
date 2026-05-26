/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::commands::app_data;
use crate::errors::UserFacingIssue;
use crate::library::{self, NexusMeta};
use crate::nexus::{
    fetch_latest_file_version, fetch_mod_details, hydrate_nexus_meta, validate_api_key,
    versions_differ, ModUpdateCheck, NexusModDetails,
};
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

    let targets: Vec<_> = library
        .mods
        .iter()
        .filter_map(|m| {
            m.nexus
                .as_ref()
                .map(|n| (m.id.clone(), m.name.clone(), n.clone()))
        })
        .collect();

    for (mod_id, mod_name, nexus) in targets {
        let remote = fetch_latest_file_version(
            &nexus.domain,
            nexus.mod_id as u64,
            nexus.file_id as u64,
            &api_key,
        )
        .await
        .map_err(|e| e.to_user_issue())?;

        let installed = nexus.version.clone();
        let (version, update_available) = match (&installed, &remote) {
            (None, Some(r)) => (Some(r.clone()), false),
            (Some(c), Some(r)) => (Some(c.clone()), versions_differ(Some(c), Some(r))),
            (Some(c), None) => (Some(c.clone()), false),
            (None, None) => (None, false),
        };

        if let Some(entry) = library.mods.iter_mut().find(|m| m.id == mod_id) {
            if let Some(meta) = &mut entry.nexus {
                if meta.version.is_none() {
                    meta.version = version.clone();
                }
                meta.update_available = update_available;
                meta.latest_version = remote.clone();
            }
        }

        if update_available {
            results.push(ModUpdateCheck {
                mod_id,
                mod_name,
                current_version: version,
                latest_version: remote,
                update_available: true,
                nexus_mod_id: nexus.mod_id as u64,
                domain: nexus.domain,
            });
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

    let updated = hydrate_nexus_meta(
        &nexus.domain,
        nexus.mod_id as u64,
        nexus.file_id as u64,
        &api_key,
        nexus.picture_url.clone(),
    )
    .await
    .map_err(|e| e.to_user_issue())?;

    let (hydrated, api_name) = updated;
    let installed_version = nexus.version.clone();
    let merged = NexusMeta {
        mod_id: nexus.mod_id,
        file_id: nexus.file_id,
        domain: nexus.domain,
        version: hydrated.version.clone().or(installed_version.clone()),
        author: hydrated.author.or(nexus.author),
        picture_url: hydrated.picture_url.or(nexus.picture_url),
        category: hydrated.category.or(nexus.category),
        endorsed: nexus.endorsed,
        tracked: nexus.tracked,
        update_available: versions_differ(
            hydrated.version.as_deref().or(installed_version.as_deref()),
            hydrated.latest_version.as_deref(),
        ),
        latest_version: hydrated.latest_version,
        summary: hydrated.summary.or(nexus.summary),
    };

    if let Some(entry) = library.mods.iter_mut().find(|m| m.id == mod_id) {
        entry.nexus = Some(merged.clone());
        if entry.name.trim().is_empty()
            || entry.name.starts_with(&format!("{} mod #", merged.domain))
        {
            entry.name = api_name;
        }
    }
    library.updated_at = library::now_ts();
    library::save_library(&data, &library).map_err(|e| e.to_user_issue())?;

    Ok(merged)
}

#[tauri::command]
pub async fn validate_nexus_api_key(_app: tauri::AppHandle) -> Result<(), UserFacingIssue> {
    let api_key = require_api_key()?;
    validate_api_key(&api_key)
        .await
        .map_err(|e| e.to_user_issue())
}
