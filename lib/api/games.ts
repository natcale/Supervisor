/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import { invoke } from "@tauri-apps/api/core";
import type { DetectedGame, GameProfileSummary, GameScanResult } from "@/types";

export async function scanGames(options?: { includeAll?: boolean }): Promise<GameScanResult> {
  // Omitting includeAll (null on the wire) uses showUnmoddableGames from saved settings.
  return invoke("scan_games", { includeAll: options?.includeAll ?? null });
}

export async function addManualGame(
  installPath: string,
  name?: string,
): Promise<DetectedGame> {
  return invoke("add_manual_game", { installPath, name: name ?? null });
}

export async function launchGame(game: DetectedGame): Promise<void> {
  return invoke("launch_game", { game });
}

export async function getGameProfile(game: DetectedGame): Promise<GameProfileSummary> {
  return invoke("get_game_profile", { game });
}

export async function listSupportedProfiles(): Promise<GameProfileSummary[]> {
  return invoke("list_supported_profiles");
}

export async function removeManualGame(gameId: string): Promise<void> {
  return invoke("remove_manual_game", { gameId });
}

export async function updateManualGameNexusDomain(
  gameId: string,
  nexusDomain: string | null,
): Promise<DetectedGame> {
  return invoke("update_manual_game_nexus_domain", {
    gameId,
    nexusDomain: nexusDomain?.trim() ? nexusDomain.trim() : null,
  });
}

export async function getStagingDir(gameId: string): Promise<string> {
  return invoke("get_staging_dir", { gameId });
}
