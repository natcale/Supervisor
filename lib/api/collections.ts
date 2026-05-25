/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import { invoke } from "@tauri-apps/api/core";
import type { CollectionModEntry } from "@/types";

export interface CollectionImportResult {
  name: string;
  gameHint?: string;
  modCount: number;
  mods: CollectionModEntry[];
}

export async function importVortexCollection(path: string): Promise<CollectionImportResult> {
  return invoke("import_vortex_collection", { path });
}

export interface CollectionInstallResult {
  queued: number;
  skipped: number;
}

/** Enqueue Nexus downloads for parsed collection mods (respects auto-start downloads setting server-side). */
export async function installCollectionMods(
  gameId: string,
  mods: CollectionModEntry[],
): Promise<CollectionInstallResult> {
  return invoke("install_collection_mods", { gameId, mods });
}
