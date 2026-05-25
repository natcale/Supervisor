/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import { invoke } from "@tauri-apps/api/core";
import type {
  DeployGameRequest,
  DeployResult,
  DeployStateResponse,
  DeployTargetSummary,
  DetectedGame,
  DiagnosticReport,
  PartitionCheckResult,
  PurgeResult,
} from "@/types";

export async function checkPartition(
  stagingDir: string,
  gameDir: string,
): Promise<PartitionCheckResult> {
  return invoke("check_partition", { stagingDir, gameDir });
}

export async function runPreflightChecks(
  gameDir: string,
  profileId: string | undefined,
  stagingDir: string,
  mods: import("@/types").ModManifest[],
  enabledIds: string[],
  conflictResolutions: Record<string, string> = {},
  deployPathOverride?: string,
): Promise<DiagnosticReport> {
  return invoke("run_preflight_checks", {
    gameDir,
    profileId: profileId ?? null,
    stagingDir,
    mods,
    enabledIds,
    conflictResolutions,
    deployPathOverride: deployPathOverride ?? null,
  });
}

export async function getDeployTargets(game: DetectedGame): Promise<DeployTargetSummary[]> {
  return invoke("get_deploy_targets", { game });
}

export async function deployGameMods(request: DeployGameRequest): Promise<DeployResult> {
  return invoke("deploy_game_mods", { request });
}

export async function getDeployState(gameId: string): Promise<DeployStateResponse | null> {
  return invoke("get_deploy_state", { gameId });
}

export async function purgeDeployedMods(gameId: string): Promise<PurgeResult> {
  return invoke("purge_deployed_mods", { gameId });
}

export async function undeployMod(gameId: string, modId: string): Promise<number> {
  return invoke("undeploy_mod", { gameId, modId });
}

export async function fixBsaTimestamps(gameDir: string): Promise<number> {
  return invoke("fix_bsa_timestamps", { gameDir });
}
