/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import { invoke } from "@tauri-apps/api/core";
import type { AppPathsInfo, AppSettings } from "@/types";

export async function getAppSettings(): Promise<AppSettings> {
  return invoke("get_app_settings");
}

export async function updateAppSettings(settings: AppSettings): Promise<AppSettings> {
  return invoke("update_app_settings", { settings });
}

export async function getAppPaths(): Promise<AppPathsInfo> {
  return invoke("get_app_paths");
}

export async function openPath(path: string): Promise<void> {
  return invoke("open_path", { path });
}

export async function completeOnboarding(): Promise<void> {
  return invoke("complete_onboarding");
}
