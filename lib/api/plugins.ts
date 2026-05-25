/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import { invoke } from "@tauri-apps/api/core";
import type { DetectedGame, ModManifest, PluginEntry } from "@/types";

export async function getPluginList(
  game: DetectedGame,
  mods: ModManifest[],
  enabledIds: string[],
): Promise<PluginEntry[]> {
  return invoke("get_plugin_list", { game, mods, enabledIds });
}

export async function sortPluginsLoot(
  game: DetectedGame,
  mods: ModManifest[],
  enabledIds: string[],
): Promise<PluginEntry[]> {
  return invoke("sort_plugins_loot", { game, mods, enabledIds });
}

export async function setPluginOrder(
  game: DetectedGame,
  mods: ModManifest[],
  enabledIds: string[],
  order: string[],
): Promise<PluginEntry[]> {
  return invoke("set_plugin_order", { game, mods, enabledIds, order });
}

export async function togglePlugin(
  game: DetectedGame,
  mods: ModManifest[],
  enabledIds: string[],
  pluginName: string,
  enabled: boolean,
): Promise<PluginEntry[]> {
  return invoke("toggle_plugin", { game, mods, enabledIds, pluginName, enabled });
}
