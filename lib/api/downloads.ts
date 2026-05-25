/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import { invoke } from "@tauri-apps/api/core";
import type { DownloadJob, NxmPayload } from "@/types";

export async function enqueueNxmDownload(
  gameId: string,
  link: {
    gameDomain: string;
    modId: number;
    fileId: number;
    key?: string;
    expires?: number;
    userId?: number;
  },
  modName?: string,
): Promise<string> {
  return invoke("enqueue_nxm_download", {
    gameId,
    link: {
      gameDomain: link.gameDomain,
      modId: link.modId,
      fileId: link.fileId,
      key: link.key ?? null,
      expires: link.expires ?? null,
      userId: link.userId ?? null,
    },
    modName: modName ?? null,
  });
}

export async function getDownloadQueue(): Promise<DownloadJob[]> {
  return invoke("get_download_queue");
}

export async function cancelDownload(jobId: string): Promise<boolean> {
  return invoke("cancel_download", { jobId });
}

export async function clearFailedDownloads(): Promise<number> {
  return invoke("clear_failed_downloads");
}

export async function clearFinishedDownloads(): Promise<number> {
  return invoke("clear_finished_downloads");
}

/** Begin processing jobs in the queued state (for manual download start). */
export async function startPendingDownloads(): Promise<number> {
  return invoke("start_pending_downloads");
}

export type { NxmPayload };
