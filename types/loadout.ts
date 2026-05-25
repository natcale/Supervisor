/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
export interface Loadout {
  id: string;
  name: string;
  enabledModIds: string[];
  conflictResolutions: Record<string, string>;
  deployPathOverride?: string;
  createdAt: number;
  updatedAt: number;
}

export interface LoadoutSummary {
  id: string;
  name: string;
  enabledCount: number;
  updatedAt: number;
}

export interface GameStateResponse {
  library: import("./mod").GameLibrary;
  loadout: Loadout;
  stagingDir: string;
}
