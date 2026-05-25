/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
export type DownloadStatus =
  | "queued"
  | "downloading"
  | "ingesting"
  | "completed"
  | "failed"
  | "cancelled";

export interface DownloadJob {
  id: string;
  gameId: string;
  gameDomain: string;
  modId: number;
  fileId: number;
  modName: string;
  pictureUrl?: string;
  status: DownloadStatus;
  progress: number;
  error?: string;
  createdAt: number;
  updatedAt: number;
}

export type NxmPayload =
  | {
      kind: "modDownload";
      gameDomain: string;
      modId: number;
      fileId: number;
      key?: string;
      expires?: number;
      userId?: number;
    }
  | { kind: "oauthCallback"; code: string; state?: string }
  | { kind: "unknown"; raw: string };
