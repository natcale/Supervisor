/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import type { UserFacingIssue } from "./mod";

export interface PartitionCheckResult {
  samePartition: boolean;
  stagingVolume: string;
  gameVolume: string;
  guidance?: UserFacingIssue;
}

export interface RootFileEntry {
  source: string;
  targetName: string;
}

export interface DeploymentRequest {
  stagingDir: string;
  gameDir: string;
  dataDir?: string;
  rootFiles: RootFileEntry[];
  modFiles: string[];
}

export interface GameProfileSummary {
  id: string;
  name: string;
  primaryModPath: string;
  isGeneric: boolean;
  supportsPlugins: boolean;
  nexusDomains?: string[];
  steamAppIds?: string[];
}

export interface ManifestTarget {
  relPath: string;
  source: string;
  modId: string;
  modType: string;
  deployRoot: string;
}

export interface DeployManifest {
  gameId: string;
  profileId: string;
  stagingPath: string;
  deployMethod: string;
  deployedAt: number;
  targets: ManifestTarget[];
}

export interface DeployReport {
  verified: boolean;
  linked: number;
  missing: number;
  mismatched: number;
  issues: UserFacingIssue[];
  profileWarning?: string;
}

export interface DeployResult {
  manifest: DeployManifest;
  report: DeployReport;
  summary: string;
  deployedFiles: number;
  profileId: string;
  profileName: string;
  primaryModPath: string;
}

export interface PersistedDeployState {
  manifest: DeployManifest;
  report: DeployReport;
  profileId: string;
  profileName: string;
  primaryModPath: string;
}

export interface DeployStateResponse {
  state: PersistedDeployState;
  driftDetected: boolean;
  checkedAt: number;
}

export interface DeployTargetSummary {
  id: string;
  label: string;
  path: string;
}

export interface PurgeResult {
  removedFiles: number;
  skipped: number;
  errors: string[];
}

export interface DeployGameRequest {
  gameId: string;
  gameDir: string;
  profileId?: string;
  stagingDir: string;
  mods: import("./mod").ModManifest[];
  enabledIds: string[];
  conflictResolutions: Record<string, string>;
  ignoreRequirements: boolean;
  deployPathOverride?: string;
}

export type UpdateCheckMode = "manual" | "onRefresh" | "onStartup";

export interface AppSettings {
  updateCheckMode: UpdateCheckMode;
  deployMethod: string;
  nexusApiKey?: string;
  /** True when credentials exist in OS keychain (masked in some responses). */
  hasNexusApiKey?: boolean;
  /** Last-selected game persisted when Remember last game is on. */
  lastGameId?: string;
  autoDeployOnChange: boolean;
  lootPath?: string;

  scanSteam: boolean;
  scanEpic: boolean;
  scanGog: boolean;
  scanHeroic: boolean;
  showUnmoddableGames: boolean;

  maxConcurrentDownloads: number;
  autoStartDownloads: boolean;
  downloadSpeedLimitKbps?: number;

  autoPurgeBeforeDeploy: boolean;
  confirmBeforeDeploy: boolean;
  verifyAfterDeploy: boolean;
  autoSortPlugins: boolean;

  showProfileWarnings: boolean;
  compactModList: boolean;
  rememberLastGame: boolean;
  alwaysShowPlugins: boolean;
  developerTools: boolean;

  collectionsSkipOptional: boolean;
  collectionsAutoEnable: boolean;

  preferScriptExtender: boolean;
  modEngineLauncherPath?: string;

  stagingRootOverride?: string;
  debugLogging: boolean;
  ignoreDeployRequirements: boolean;
  activeThemeId?: string;
  /** Show the compact game icon bar beside the main sidebar. */
  compactGameSidebar?: boolean;
  /** When true, hides the compact bar even if a theme requests it. */
  compactGameSidebarHidden?: boolean;
}

export interface AppPathsInfo {
  appDataDir: string;
  stagingRoot: string;
  downloadsDir: string;
  themesDir: string;
}

