/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::errors::{AppError, AppResult};
use crate::library::now_ts;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DownloadStatus {
    Queued,
    Downloading,
    Ingesting,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadJob {
    pub id: String,
    pub game_id: String,
    pub game_domain: String,
    pub mod_id: u64,
    pub file_id: u64,
    pub mod_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub picture_url: Option<String>,
    pub status: DownloadStatus,
    pub progress: u8,
    pub error: Option<String>,
    /// Ephemeral NXM authorization from Mod Manager Download (free users).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nxm_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nxm_expires: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nxm_user_id: Option<u64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Default)]
pub struct DownloadQueue {
    inner: Mutex<Vec<DownloadJob>>,
    app_data: Mutex<Option<PathBuf>>,
}

impl DownloadQueue {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Vec::new()),
            app_data: Mutex::new(None),
        }
    }

    pub fn init(&self, app_data: &Path) -> AppResult<()> {
        *self.app_data.lock().unwrap() = Some(app_data.to_path_buf());
        if let Ok(mut jobs) = load_persisted_queue(app_data) {
            jobs.retain(|j| !matches!(j.status, DownloadStatus::Completed));
            recover_stale_jobs(&mut jobs);
            normalize_jobs(&mut jobs);
            *self.inner.lock().unwrap() = jobs;
            self.persist();
        }
        Ok(())
    }

    pub fn list(&self) -> Vec<DownloadJob> {
        let mut jobs = self.inner.lock().unwrap();
        let before = jobs.len();
        normalize_jobs(&mut jobs);
        let result = jobs.clone();
        let changed = jobs.len() != before;
        drop(jobs);
        if changed {
            self.persist();
        }
        result
    }

    pub fn enqueue(&self, job: DownloadJob) {
        let mut jobs = self.inner.lock().unwrap();
        purge_inactive_duplicates_for(&mut jobs, &job.game_id, job.mod_id, job.file_id);
        jobs.push(job);
        normalize_jobs(&mut jobs);
        drop(jobs);
        self.persist();
    }

    pub fn update(&self, id: &str, update: impl FnOnce(&mut DownloadJob)) {
        let mut jobs = self.inner.lock().unwrap();
        if let Some(job) = jobs.iter_mut().find(|j| j.id == id) {
            update(job);
            job.updated_at = now_ts();
        }
        drop(jobs);
        self.persist();
    }

    pub fn remove(&self, id: &str) -> bool {
        let mut jobs = self.inner.lock().unwrap();
        let before = jobs.len();
        jobs.retain(|j| j.id != id);
        let removed = jobs.len() < before;
        drop(jobs);
        if removed {
            self.persist();
        }
        removed
    }

    pub fn cancel(&self, id: &str) -> bool {
        let mut jobs = self.inner.lock().unwrap();
        let mut changed = false;
        if let Some(job) = jobs.iter_mut().find(|j| j.id == id) {
            if matches!(
                job.status,
                DownloadStatus::Queued | DownloadStatus::Downloading
            ) {
                job.status = DownloadStatus::Cancelled;
                job.updated_at = now_ts();
                changed = true;
            }
        }
        drop(jobs);
        if changed {
            self.persist();
        }
        changed
    }

    pub fn has_active_job(&self, game_id: &str, mod_id: u64, file_id: u64) -> bool {
        self.inner.lock().unwrap().iter().any(|j| {
            j.game_id == game_id
                && j.mod_id == mod_id
                && j.file_id == file_id
                && matches!(
                    j.status,
                    DownloadStatus::Queued
                        | DownloadStatus::Downloading
                        | DownloadStatus::Ingesting
                )
        })
    }

    pub fn clear_failed(&self) -> usize {
        let mut jobs = self.inner.lock().unwrap();
        let before = jobs.len();
        jobs.retain(|j| j.status != DownloadStatus::Failed);
        normalize_jobs(&mut jobs);
        let removed = before - jobs.len();
        drop(jobs);
        if removed > 0 {
            self.persist();
        }
        removed
    }

    pub fn clear_finished(&self) -> usize {
        let mut jobs = self.inner.lock().unwrap();
        let before = jobs.len();
        jobs.retain(|j| is_active_status(&j.status));
        normalize_jobs(&mut jobs);
        let removed = before - jobs.len();
        drop(jobs);
        if removed > 0 {
            self.persist();
        }
        removed
    }

    fn persist(&self) {
        let app_data = self.app_data.lock().unwrap().clone();
        let Some(app_data) = app_data else {
            return;
        };
        let jobs = self.inner.lock().unwrap().clone();
        let _ = persist_queue(&app_data, &jobs);
    }
}

fn queue_path(app_data: &Path) -> PathBuf {
    app_data.join("downloads").join("queue.json")
}

pub fn persist_queue(app_data: &Path, jobs: &[DownloadJob]) -> AppResult<()> {
    let path = queue_path(app_data);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(AppError::Io)?;
    }
    let raw = serde_json::to_string_pretty(jobs).map_err(|e| AppError::user(e.to_string()))?;
    fs::write(path, raw).map_err(AppError::Io)
}

pub fn load_persisted_queue(app_data: &Path) -> AppResult<Vec<DownloadJob>> {
    let path = queue_path(app_data);
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&path).map_err(AppError::Io)?;
    serde_json::from_str(&raw).map_err(|e| AppError::user(format!("Corrupt download queue: {e}")))
}

fn is_active_status(status: &DownloadStatus) -> bool {
    matches!(
        status,
        DownloadStatus::Queued | DownloadStatus::Downloading | DownloadStatus::Ingesting
    )
}

fn job_key(job: &DownloadJob) -> (String, u64, u64) {
    (job.game_id.clone(), job.mod_id, job.file_id)
}

fn purge_inactive_duplicates_for(
    jobs: &mut Vec<DownloadJob>,
    game_id: &str,
    mod_id: u64,
    file_id: u64,
) {
    jobs.retain(|j| {
        !(j.game_id == game_id
            && j.mod_id == mod_id
            && j.file_id == file_id
            && !is_active_status(&j.status))
    });
}

fn recover_stale_jobs(jobs: &mut [DownloadJob]) {
    for job in jobs.iter_mut() {
        if job.status == DownloadStatus::Downloading {
            job.status = DownloadStatus::Queued;
            job.progress = 0;
            job.updated_at = now_ts();
        }
    }
}

fn normalize_jobs(jobs: &mut Vec<DownloadJob>) {
    let mut best: std::collections::HashMap<(String, u64, u64), DownloadJob> =
        std::collections::HashMap::new();
    for job in jobs.drain(..) {
        let key = job_key(&job);
        match best.get(&key) {
            None => {
                best.insert(key, job);
            }
            Some(existing) => {
                let keep_new =
                    if is_active_status(&job.status) && !is_active_status(&existing.status) {
                        true
                    } else if is_active_status(&existing.status) && !is_active_status(&job.status) {
                        false
                    } else {
                        job.updated_at > existing.updated_at
                    };
                if keep_new {
                    best.insert(key, job);
                }
            }
        }
    }
    *jobs = best.into_values().collect();
    jobs.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
}
