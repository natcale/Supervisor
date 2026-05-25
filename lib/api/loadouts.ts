/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import { invoke } from "@tauri-apps/api/core";
import type { Loadout, LoadoutSummary } from "@/types";

export async function listLoadouts(
  gameId: string,
): Promise<[LoadoutSummary[], string]> {
  return invoke("list_loadouts", { gameId });
}

export async function switchLoadout(gameId: string, loadoutId: string): Promise<Loadout> {
  return invoke("switch_loadout", { gameId, loadoutId });
}

export async function createLoadout(gameId: string, name: string): Promise<Loadout> {
  return invoke("create_loadout", { gameId, name });
}

export async function updateLoadout(gameId: string, loadout: Loadout): Promise<Loadout> {
  return invoke("update_loadout", { gameId, loadout });
}

export async function deleteLoadout(gameId: string, loadoutId: string): Promise<void> {
  return invoke("delete_loadout", { gameId, loadoutId });
}
