/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::commands::{app_data, persist_ingested};
use crate::errors::{AppError, UserFacingIssue};
use crate::ingest;
use crate::library::{self, NexusMeta};
use crate::nexus::{self, DownloadJob, DownloadQueue, DownloadStatus};
use crate::secrets;
use crate::settings;
use crate::AppState;
use std::sync::Arc;
use tauri::{Emitter, Manager, State};

#[tauri::command]
pub fn get_download_queue(state: State<'_, AppState>) -> Vec<DownloadJob> {
    state.downloads.list()
}

fn emit_queue_changed(app: &tauri::AppHandle, queue: &DownloadQueue) {
    let _ = app.emit("download://queue-changed", queue.list());
}

#[tauri::command]
pub fn cancel_download(app: tauri::AppHandle, state: State<'_, AppState>, job_id: String) -> bool {
    let changed = state.downloads.cancel(&job_id);
    if changed {
        emit_queue_changed(&app, &state.downloads);
    }
    changed
}

#[tauri::command]
pub fn clear_failed_downloads(app: tauri::AppHandle, state: State<'_, AppState>) -> usize {
    let removed = state.downloads.clear_failed();
    if removed > 0 {
        emit_queue_changed(&app, &state.downloads);
    }
    removed
}

#[tauri::command]
pub fn clear_finished_downloads(app: tauri::AppHandle, state: State<'_, AppState>) -> usize {
    let removed = state.downloads.clear_finished();
    if removed > 0 {
        emit_queue_changed(&app, &state.downloads);
    }
    removed
}

#[tauri::command]
pub fn start_pending_downloads(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<usize, UserFacingIssue> {
    Ok(pump_download_queue(&app, &state.downloads))
}

#[tauri::command]
pub fn parse_nxm(raw_url: String) -> Option<crate::deep_link::NxmPayload> {
    crate::deep_link::parse_nxm_url(&raw_url)
}

#[tauri::command]
pub async fn enqueue_nxm_download(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    game_id: String,
    link: crate::deep_link::NxmModLink,
    mod_name: Option<String>,
) -> Result<String, UserFacingIssue> {
    let api_key = secrets::get_nexus_api_key().map_err(|e| e.to_user_issue())?;
    if link.key.is_none() && api_key.as_ref().map(|k| k.trim().is_empty()).unwrap_or(true) {
        return Err(AppError::user(
            "This download link is missing authorization. Add a Nexus API key in Settings, or click \
             \"Mod Manager Download\" on Nexus while logged in.",
        )
        .to_user_issue());
    }

    if state
        .downloads
        .has_active_job(&game_id, link.mod_id, link.file_id)
    {
        let existing = state
            .downloads
            .list()
            .into_iter()
            .find(|j| {
                j.game_id == game_id
                    && j.mod_id == link.mod_id
                    && j.file_id == link.file_id
                    && matches!(
                        j.status,
                        DownloadStatus::Queued
                            | DownloadStatus::Downloading
                            | DownloadStatus::Ingesting
                    )
            });
        if let Some(job) = existing {
            return Ok(job.id);
        }
    }

    let mut name = mod_name.unwrap_or_else(|| format!("{} mod #{}", link.game_domain, link.mod_id));
    let mut picture_url = None;
    if let Some(ref key) = api_key {
        if let Ok(details) = nexus::fetch_mod_details(&link.game_domain, link.mod_id, key).await {
            name = details.name;
            picture_url = details.picture_url;
        }
    }

    let job_id = format!("dl-{}", uuid::Uuid::new_v4());
    let job = DownloadJob {
        id: job_id.clone(),
        game_id: game_id.clone(),
        game_domain: link.game_domain.clone(),
        mod_id: link.mod_id,
        file_id: link.file_id,
        mod_name: name,
        picture_url,
        status: DownloadStatus::Queued,
        progress: 0,
        error: None,
        nxm_key: link.key.clone(),
        nxm_expires: link.expires,
        nxm_user_id: link.user_id,
        created_at: library::now_ts(),
        updated_at: library::now_ts(),
    };
    state.downloads.enqueue(job.clone());
    let _ = app.emit("download://updated", &job);

    let data = app_data(&app)?;
    let app_settings = settings::load_settings(&data).map_err(|e| e.to_user_issue())?;
    if app_settings.auto_start_downloads {
        pump_download_queue(&app, &state.downloads);
    }

    Ok(job_id)
}

#[tauri::command]
pub async fn download_nxm_mod(
    app: tauri::AppHandle,
    game_id: String,
    link: crate::deep_link::NxmModLink,
    mod_name: Option<String>,
) -> Result<serde_json::Value, UserFacingIssue> {
    let data = app_data(&app)?;
    let app_settings = settings::load_settings(&data).map_err(|e| e.to_user_issue())?;
    let api_key = secrets::get_nexus_api_key().map_err(|e| e.to_user_issue())?;

    let staging = settings::game_staging_dir(&data, &app_settings, &game_id).join("downloads");
    std::fs::create_dir_all(&staging).map_err(|e| AppError::Io(e).to_user_issue())?;

    let archive = nexus::download_mod_archive(
        &link,
        &staging,
        api_key.as_deref(),
        app_settings.download_speed_limit_kbps,
    )
    .await
    .map_err(|e| e.to_user_issue())?;

    let game_staging = staging.parent().unwrap().to_path_buf();
    let mut result = ingest::ingest_paths(&game_staging, &[archive.to_string_lossy().into_owned()])
        .map_err(|e| e.to_user_issue())?;

    if let Some(m) = result.mods.first_mut() {
        m.id = format!("nxm-{}-{}", link.mod_id, link.file_id);
        m.name = mod_name.unwrap_or_else(|| format!("{} mod #{}", link.game_domain, link.mod_id));
        m.nexus = Some(NexusMeta {
            mod_id: link.mod_id,
            file_id: link.file_id,
            domain: link.game_domain.clone(),
            version: None,
            author: None,
            picture_url: None,
            category: None,
            endorsed: None,
            tracked: false,
            update_available: false,
            latest_version: None,
            summary: None,
        });
    }

    persist_ingested(&data, &game_id, &result.mods)?;
    Ok(serde_json::to_value(result).unwrap())
}

pub fn pump_download_queue(app: &tauri::AppHandle, queue: &Arc<DownloadQueue>) -> usize {
    let data = match app_data(app) {
        Ok(d) => d,
        Err(_) => return 0,
    };
    let app_settings = settings::load_settings(&data).unwrap_or_default();
    let max = app_settings.max_concurrent_downloads.max(1) as usize;
    let active = queue
        .list()
        .iter()
        .filter(|j| {
            matches!(
                j.status,
                DownloadStatus::Downloading | DownloadStatus::Ingesting
            )
        })
        .count();
    if active >= max {
        return 0;
    }

    let mut started = 0usize;
    for job in queue.list() {
        if active + started >= max {
            break;
        }
        if job.status != DownloadStatus::Queued {
            continue;
        }
        let Some(link) = job_to_link(&job) else {
            continue;
        };
        let app_handle = app.clone();
        let downloads = queue.clone();
        let job_id = job.id.clone();
        let game_id = job.game_id.clone();
        let mod_name = job.mod_name.clone();
        let speed_limit = app_settings.download_speed_limit_kbps;
        queue.update(&job_id, |j| {
            j.status = DownloadStatus::Downloading;
            j.progress = 5;
        });
        emit_job_update(queue, app, &job_id);
        tauri::async_runtime::spawn(async move {
            run_download_job(
                app_handle,
                downloads,
                game_id,
                link,
                mod_name,
                job_id,
                speed_limit,
            )
            .await;
        });
        started += 1;
    }
    started
}

fn job_to_link(job: &DownloadJob) -> Option<crate::deep_link::NxmModLink> {
    Some(crate::deep_link::NxmModLink {
        game_domain: job.game_domain.clone(),
        mod_id: job.mod_id,
        file_id: job.file_id,
        key: job.nxm_key.clone(),
        expires: job.nxm_expires,
        user_id: job.nxm_user_id,
    })
}

fn emit_job_update(queue: &DownloadQueue, app: &tauri::AppHandle, job_id: &str) {
    if let Some(job) = queue.list().into_iter().find(|j| j.id == job_id) {
        let _ = app.emit("download://updated", &job);
    }
}

async fn run_download_job(
    app: tauri::AppHandle,
    queue: Arc<DownloadQueue>,
    game_id: String,
    link: crate::deep_link::NxmModLink,
    mod_name: String,
    job_id: String,
    speed_limit_kbps: Option<u32>,
) {
    let emit_update = |queue: &DownloadQueue, app: &tauri::AppHandle| {
        emit_job_update(queue, app, &job_id);
    };

    let data = match app.path().app_data_dir() {
        Ok(d) => d,
        Err(e) => {
            queue.update(&job_id, |j| {
                j.status = DownloadStatus::Failed;
                j.error = Some(e.to_string());
            });
            emit_update(&queue, &app);
            pump_download_queue(&app, &queue);
            return;
        }
    };

    let app_settings = settings::load_settings(&data).unwrap_or_default();
    let dl_dir = settings::game_staging_dir(&data, &app_settings, &game_id).join("downloads");
    if std::fs::create_dir_all(&dl_dir).is_err() {
        queue.update(&job_id, |j| {
            j.status = DownloadStatus::Failed;
            j.error = Some("Could not create download folder".into());
        });
        emit_update(&queue, &app);
        pump_download_queue(&app, &queue);
        return;
    }

    let api_key = secrets::get_nexus_api_key().ok().flatten();
    let archive = match nexus::download_mod_archive(
        &link,
        &dl_dir,
        api_key.as_deref(),
        speed_limit_kbps,
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            queue.update(&job_id, |j| {
                j.status = DownloadStatus::Failed;
                j.error = Some(e.to_string());
            });
            emit_update(&queue, &app);
            pump_download_queue(&app, &queue);
            return;
        }
    };

    queue.update(&job_id, |j| {
        j.status = DownloadStatus::Ingesting;
        j.progress = 80;
    });
    emit_update(&queue, &app);

    let game_staging = dl_dir.parent().unwrap().to_path_buf();
    let ingest_result = match ingest::ingest_paths(&game_staging, &[archive.to_string_lossy().into_owned()]) {
        Ok(r) => r,
        Err(e) => {
            queue.update(&job_id, |j| {
                j.status = DownloadStatus::Failed;
                j.error = Some(e.to_string());
            });
            emit_update(&queue, &app);
            pump_download_queue(&app, &queue);
            return;
        }
    };

    let mut ingested = ingest_result.mods;
    if let Some(m) = ingested.first_mut() {
        m.id = format!("nxm-{}-{}", link.mod_id, link.file_id);
        m.name = mod_name;
        m.nexus = Some(NexusMeta {
            mod_id: link.mod_id,
            file_id: link.file_id,
            domain: link.game_domain.clone(),
            version: None,
            author: None,
            picture_url: None,
            category: None,
            endorsed: None,
            tracked: false,
            update_available: false,
            latest_version: None,
            summary: None,
        });
    }

    if let Err(e) = persist_ingested(&data, &game_id, &ingested) {
        queue.update(&job_id, |j| {
            j.status = DownloadStatus::Failed;
            j.error = Some(e.title);
        });
        emit_update(&queue, &app);
        pump_download_queue(&app, &queue);
        return;
    }

    let picture_url = queue
        .list()
        .iter()
        .find(|j| j.id == job_id)
        .and_then(|j| j.picture_url.clone());

    if let Some(m) = ingested.first_mut() {
        if let Some(ref url) = picture_url {
            if let Some(ref mut nexus) = m.nexus {
                nexus.picture_url = Some(url.clone());
            }
        }
    }

    queue.remove(&job_id);
    let _ = app.emit(
        "download://completed",
        &serde_json::json!({
            "jobId": job_id,
            "gameId": game_id,
            "mods": ingested,
            "stagingDir": ingest_result.staging_dir,
        }),
    );
    pump_download_queue(&app, &queue);
}
