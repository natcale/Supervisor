/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import { invoke } from "@tauri-apps/api/core";
import type { ModUpdateInfo, NexusModMetadata } from "@/types";

export interface NexusModDetails {
  modId: number;
  name: string;
  summary?: string;
  pictureUrl?: string;
  author?: string;
  version?: string;
  domain: string;
  category?: string;
}

export async function fetchNexusModMetadata(
  domain: string,
  modId: number,
): Promise<NexusModDetails> {
  return invoke("fetch_nexus_mod_metadata", { domain, modId });
}

export async function checkModUpdates(gameId: string): Promise<
  Array<{
    modId: string;
    modName: string;
    currentVersion?: string;
    latestVersion?: string;
    updateAvailable: boolean;
    nexusModId: number;
    domain: string;
  }>
> {
  return invoke("check_mod_updates", { gameId });
}

export async function enrichModMetadata(
  gameId: string,
  modId: string,
): Promise<NexusModMetadata & { modId: number; fileId: number; domain: string }> {
  return invoke("enrich_mod_metadata", { gameId, modId });
}

export async function validateNexusApiKey(): Promise<void> {
  return invoke("validate_nexus_api_key");
}

export type { ModUpdateInfo };
