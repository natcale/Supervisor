/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import type { DownloadJob, DownloadStatus } from "@/types";

export function isActiveDownloadStatus(status: DownloadStatus): boolean {
  return status === "queued" || status === "downloading" || status === "ingesting";
}

export function isActiveDownload(job: DownloadJob): boolean {
  return isActiveDownloadStatus(job.status);
}

/** Keep the newest job per mod file. Prefer active jobs over finished duplicates. */
export function dedupeDownloadJobs(jobs: DownloadJob[]): DownloadJob[] {
  const map = new Map<string, DownloadJob>();
  for (const job of jobs) {
    const key = `${job.gameId}:${job.modId}:${job.fileId}`;
    const existing = map.get(key);
    if (!existing) {
      map.set(key, job);
      continue;
    }
    const jobActive = isActiveDownload(job);
    const existingActive = isActiveDownload(existing);
    if (jobActive && !existingActive) {
      map.set(key, job);
      continue;
    }
    if (!jobActive && existingActive) {
      continue;
    }
    if (job.updatedAt > existing.updatedAt) {
      map.set(key, job);
    }
  }
  return Array.from(map.values()).sort((a, b) => b.updatedAt - a.updatedAt);
}

export function nexusModThumbUrl(gameDomain: string, modId: number): string {
  return `https://staticdelivery.nexusmods.com/mods/${modId}/images/thumbnail.jpg?domain=${gameDomain}`;
}

export function downloadJobImage(job: DownloadJob): string | undefined {
  return job.pictureUrl ?? nexusModThumbUrl(job.gameDomain, job.modId);
}

export function statusLabel(status: DownloadStatus): string {
  switch (status) {
    case "queued":
      return "queued";
    case "downloading":
      return "downloading";
    case "ingesting":
      return "installing";
    case "completed":
      return "completed";
    case "failed":
      return "failed";
    case "cancelled":
      return "cancelled";
    default:
      return status;
  }
}
