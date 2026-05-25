/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use crate::collections::parse_collection_file;
use crate::commands::{app_data, downloads::pump_download_queue};
use crate::errors::UserFacingIssue;
use crate::library;
use crate::nexus::{DownloadJob, DownloadStatus};
use crate::settings;
use crate::AppState;
use tauri::{Emitter, State};

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionImportResult {
    pub name: String,
    pub game_hint: Option<String>,
    pub mod_count: usize,
    pub mods: Vec<crate::collections::CollectionModEntry>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionInstallResult {
    pub queued: usize,
    pub skipped: usize,
}

#[tauri::command]
pub fn import_vortex_collection(
    app: tauri::AppHandle,
    path: String,
) -> Result<CollectionImportResult, UserFacingIssue> {
    let _ = app_data(&app)?;
    let parsed = parse_collection_file(std::path::Path::new(&path)).map_err(|e| e.to_user_issue())?;
    Ok(CollectionImportResult {
        name: parsed.name,
        game_hint: parsed.game_hint,
        mod_count: parsed.mods.len(),
        mods: parsed.mods,
    })
}

#[tauri::command]
pub async fn install_collection_mods(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    game_id: String,
    mods: Vec<crate::collections::CollectionModEntry>,
) -> Result<CollectionInstallResult, UserFacingIssue> {
    let data = app_data(&app)?;
    let app_settings = settings::load_settings(&data).map_err(|e| e.to_user_issue())?;
    let mut queued = 0usize;
    let mut skipped = 0usize;

    for entry in mods {
        if app_settings.collections_skip_optional && entry.optional.unwrap_or(false) {
            skipped += 1;
            continue;
        }
        let (Some(mod_id), Some(file_id), Some(domain)) =
            (entry.mod_id, entry.file_id, entry.domain_name.as_ref())
        else {
            skipped += 1;
            continue;
        };

        if state.downloads.has_active_job(&game_id, mod_id, file_id) {
            skipped += 1;
            continue;
        }

        let job_id = format!("dl-{}", uuid::Uuid::new_v4());
        let job = DownloadJob {
            id: job_id.clone(),
            game_id: game_id.clone(),
            game_domain: domain.clone(),
            mod_id,
            file_id,
            mod_name: entry.name.clone(),
            picture_url: None,
            status: DownloadStatus::Queued,
            progress: 0,
            error: None,
            nxm_key: None,
            nxm_expires: None,
            nxm_user_id: None,
            created_at: library::now_ts(),
            updated_at: library::now_ts(),
        };
        state.downloads.enqueue(job.clone());
        let _ = app.emit("download://updated", &job);
        queued += 1;
    }

    if app_settings.auto_start_downloads {
        pump_download_queue(&app, &state.downloads);
    }

    Ok(CollectionInstallResult { queued, skipped })
}
