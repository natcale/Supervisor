/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import { invoke } from "@tauri-apps/api/core";
import type {
  FomodConfig,
  GameLibrary,
  GameStateResponse,
  IngestResult,
  ModManifest,
} from "@/types";

function mapIngested(raw: Record<string, unknown>): ModManifest {
  return {
    id: String(raw.id),
    name: String(raw.name),
    slug: raw.slug as string | undefined,
    files: (raw.files as string[]) ?? [],
    dependencies: (raw.dependencies as string[]) ?? [],
    installState: raw.installState as ModManifest["installState"],
    needsFomod: Boolean(raw.needsFomod),
    nexus: raw.nexus as ModManifest["nexus"],
    notes: raw.notes as string | undefined,
  };
}

export async function ingestModPaths(gameId: string, paths: string[]): Promise<IngestResult> {
  const result = await invoke<{ mods: Record<string, unknown>[]; stagingDir: string }>(
    "ingest_mod_paths",
    { gameId, paths },
  );
  return {
    mods: result.mods.map(mapIngested),
    stagingDir: result.stagingDir,
  };
}

export async function getGameState(gameId: string): Promise<GameStateResponse> {
  return invoke("get_game_state", { gameId });
}

export async function getLibrary(gameId: string): Promise<GameLibrary> {
  return invoke("get_library", { gameId });
}

export async function removeLibraryMod(gameId: string, modId: string): Promise<GameLibrary> {
  return invoke("remove_library_mod", { gameId, modId });
}

export async function reorderLibraryMods(
  gameId: string,
  modIds: string[],
): Promise<GameLibrary> {
  return invoke("reorder_library_mods", { gameId, modIds });
}

export async function parseFomod(gameId: string, slug: string): Promise<FomodConfig> {
  return invoke("parse_fomod", { gameId, slug });
}

export async function applyFomod(
  gameId: string,
  modId: string,
  slug: string,
  selections: string[],
): Promise<ModManifest> {
  const raw = await invoke<Record<string, unknown>>("apply_fomod", {
    gameId,
    modId,
    slug,
    selections,
  });
  return mapIngested(raw);
}

export async function reinstallMod(gameId: string, modId: string): Promise<ModManifest> {
  const raw = await invoke<Record<string, unknown>>("reinstall_mod", { gameId, modId });
  return mapIngested(raw);
}

export function libraryModToManifest(mod: import("@/types").LibraryMod): ModManifest {
  return {
    id: mod.id,
    name: mod.name,
    files: mod.files,
    dependencies: mod.dependencies,
    slug: mod.slug,
    installState: mod.installState,
    needsFomod: mod.installState === "pendingFomod",
    nexus: mod.nexus,
    notes: mod.notes,
  };
}

export function ingestedToManifest(mod: ModManifest): ModManifest {
  return mod;
}

export async function openModFolder(
  gameId: string,
  modId: string,
  slug?: string,
): Promise<void> {
  return invoke("open_mod_folder", { gameId, modId, slug: slug ?? null });
}

export async function openStagingFolder(gameId: string): Promise<void> {
  return invoke("open_staging_folder", { gameId });
}

export async function setModNotes(
  gameId: string,
  modId: string,
  notes: string | null,
): Promise<GameLibrary> {
  return invoke("set_mod_notes", { gameId, modId, notes });
}
