/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
export interface UserChoice {
  id: string;
  label: string;
  description: string;
  recommended: boolean;
}

export interface UserFacingIssue {
  id: string;
  title: string;
  explanation: string;
  impact: string;
  choices: UserChoice[];
}

export interface DiagnosticReport {
  ready: boolean;
  issues: UserFacingIssue[];
  summary: string;
}

export type EndorsementState = "undecided" | "endorsed" | "abstained";

export interface NexusMeta {
  modId: number;
  fileId: number;
  domain: string;
  version?: string;
  author?: string;
  pictureUrl?: string;
  category?: string;
  endorsed?: EndorsementState;
  tracked?: boolean;
  updateAvailable?: boolean;
  latestVersion?: string;
  summary?: string;
}

export interface ModManifest {
  id: string;
  name: string;
  files: string[];
  dependencies: string[];
  slug?: string;
  installState?: "pendingFomod" | "installed";
  needsFomod?: boolean;
  nexus?: NexusMeta;
  notes?: string;
}

export interface LibraryMod {
  id: string;
  name: string;
  slug: string;
  files: string[];
  dependencies: string[];
  installState: "pendingFomod" | "installed";
  installedAt: number;
  nexus?: NexusMeta;
  notes?: string;
}

export interface GameLibrary {
  gameId: string;
  mods: LibraryMod[];
  updatedAt: number;
}

export interface IngestResult {
  mods: ModManifest[];
  stagingDir: string;
}

export interface FomodOption {
  id: string;
  name: string;
  description: string;
  optionType: string;
  isDefault: boolean;
}

export interface FomodStep {
  id: string;
  name: string;
  options: FomodOption[];
}

export interface FomodConfig {
  moduleName: string;
  steps: FomodStep[];
}

export interface NexusModMetadata {
  modId: number;
  name: string;
  summary?: string;
  pictureUrl?: string;
  author?: string;
  version?: string;
  domain: string;
  category?: string;
  endorsed?: EndorsementState;
}

export interface ModUpdateInfo {
  modId: string;
  updateAvailable: boolean;
  latestVersion?: string;
}

export interface ModTableColumn {
  id: string;
  label: string;
  visible: boolean;
  width?: string;
}

export const DEFAULT_MOD_COLUMNS: ModTableColumn[] = [
  { id: "enabled", label: "Enabled", visible: true, width: "100px" },
  { id: "name", label: "Mod Name", visible: true },
  { id: "version", label: "Version", visible: true, width: "120px" },
  { id: "author", label: "Author", visible: true, width: "140px" },
  { id: "category", label: "Category", visible: true, width: "120px" },
  { id: "actions", label: "Actions", visible: true, width: "120px" },
];

export interface PluginEntry {
  name: string;
  modId: string;
  enabled: boolean;
  isMaster: boolean;
}

export interface CollectionModEntry {
  name: string;
  version?: string;
  domainName?: string;
  modId?: number;
  fileId?: number;
  optional?: boolean;
}

export interface GameQueue {
  mods: ModManifest[];
  enabledIds: string[];
  stagingDir: string;
  conflictResolutions: Record<string, string>;
  deployPathOverride?: string;
  activeLoadoutId?: string;
  loadoutName?: string;
}

export type GameQueues = Record<string, GameQueue>;

export function emptyQueue(): GameQueue {
  return { mods: [], enabledIds: [], stagingDir: "", conflictResolutions: {} };
}
