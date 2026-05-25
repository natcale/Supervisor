/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import type { AppSettings } from "@/types";

export const SETTINGS_DEFAULTS: AppSettings = {
  updateCheckMode: "onRefresh",
  deployMethod: "hardlink",
  autoDeployOnChange: true,
  scanSteam: true,
  scanEpic: true,
  scanGog: true,
  scanHeroic: true,
  showUnmoddableGames: false,
  maxConcurrentDownloads: 2,
  autoStartDownloads: true,
  autoPurgeBeforeDeploy: false,
  confirmBeforeDeploy: false,
  verifyAfterDeploy: true,
  autoSortPlugins: true,
  showProfileWarnings: true,
  compactModList: false,
  rememberLastGame: true,
  alwaysShowPlugins: false,
  developerTools: false,
  hasNexusApiKey: false,
  collectionsSkipOptional: false,
  collectionsAutoEnable: true,
  preferScriptExtender: true,
  debugLogging: false,
  ignoreDeployRequirements: false,
  activeThemeId: "default",
  compactGameSidebar: false,
  compactGameSidebarHidden: false,
};

export function mergeSettings(raw: AppSettings): AppSettings {
  return { ...SETTINGS_DEFAULTS, ...raw };
}
