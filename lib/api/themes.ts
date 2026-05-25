/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import { invoke } from "@tauri-apps/api/core";
import type { LoadedTheme, ThemeSummary } from "@/types";

export async function listThemes(): Promise<ThemeSummary[]> {
  return invoke("list_themes");
}

export async function loadActiveTheme(): Promise<LoadedTheme> {
  return invoke("load_active_theme");
}

export async function setActiveTheme(themeId: string): Promise<LoadedTheme> {
  return invoke("set_active_theme", { themeId });
}

export async function installTheme(archivePath: string): Promise<ThemeSummary> {
  return invoke("install_theme", { archivePath });
}

export async function getPlatform(): Promise<string> {
  return invoke("get_platform");
}

export async function openThemesFolder(): Promise<void> {
  return invoke("open_themes_folder");
}

export async function readThemeAsset(themeId: string, relativePath: string): Promise<Uint8Array> {
  return invoke("read_theme_asset", { themeId, relativePath });
}
