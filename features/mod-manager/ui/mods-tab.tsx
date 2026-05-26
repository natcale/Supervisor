/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { IssueModal } from "@/components/ui/issue-modal";
import { usePreflight } from "@/features/mod-manager/model/use-preflight";
import { FomodWizard } from "@/features/mod-tools/ui/fomod-wizard";
import { PluginList } from "@/features/mod-tools/ui/plugin-list";
import { ModBatchBar } from "@/features/mod-manager/ui/mod-batch-bar";
import { ConflictPanel } from "@/features/mod-manager/ui/conflict-panel";
import {
  applyModFilter,
  ModFilterBar,
  type ModFilter,
} from "@/features/mod-manager/ui/mod-filter-bar";
import { ModDropzone } from "@/features/mod-manager/ui/mod-dropzone";
import { DeployAdvisoryBanner } from "@/features/mod-manager/ui/deploy-advisory-banner";
import { ModInfoPanel } from "@/features/mod-manager/ui/mod-info-panel";
import { ModToolbar } from "@/features/mod-manager/ui/mod-toolbar";
import {
  conflictModIdsFromIssues,
  ModTable,
} from "@/features/mod-manager/ui/mod-table";
import {
  loadModColumns,
  saveModColumns,
  toggleColumn,
} from "@/features/mod-manager/model/mod-table-columns";
import { ingestForGame, pickModPaths } from "@/lib/drop";
import {
  applyFomod,
  getGameState,
  libraryModToManifest,
  openStagingFolder,
  parseFomod,
  reinstallMod,
  reorderLibraryMods,
} from "@/lib/api/mods";
import {
  checkPartition,
  deployGameMods,
  fixBsaTimestamps,
  getDeployState,
  purgeDeployedMods,
} from "@/lib/api/deploy";
import { getGameProfile } from "@/lib/api/games";
import { checkModUpdates } from "@/lib/api/nexus";
import { enqueueNxmDownload } from "@/lib/api/downloads";
import { getAppSettings } from "@/lib/api/settings";
import { openPath } from "@/lib/api/settings";
import { mergeSettings } from "@/lib/settings-defaults";
import { isTauri } from "@/lib/env";
import { openPathSafe } from "@/lib/errors";
import { listen } from "@tauri-apps/api/event";
import type {
  AppSettings,
  DetectedGame,
  FomodConfig,
  GameQueue,
  Loadout,
  ModManifest,
  UserFacingIssue,
  ShellView,
} from "@/types";

type Props = {
  game: DetectedGame;
  queue: GameQueue;
  deployRefreshKey?: number;
  onToggle: (modId: string) => void;
  onRemove: (modId: string) => void;
  onIngested: (mods: ModManifest[], stagingDir: string) => void;
  onConflictChoice: (issueId: string, choiceId: string) => void;
  onEnableMod: (modId: string) => void;
  onDeployComplete: () => void;
  onLoadoutChange: (loadout: Loadout) => void;
  onModsUpdated: (mods: ModManifest[]) => void;
  onModsReordered?: (mods: ModManifest[]) => void;
  onDeployPathOverride: (path: string | undefined) => void;
  ignoreRequirements: boolean;
  onIgnoreRequirements: () => void;
  defaultShowPlugins?: boolean;
  onNavigate?: (view: ShellView) => void;
};

export function ModsTab({
  game,
  queue,
  onToggle,
  onRemove,
  onIngested,
  onConflictChoice,
  onEnableMod,
  onDeployComplete,
  onLoadoutChange,
  onModsUpdated,
  onModsReordered,
  onDeployPathOverride,
  deployRefreshKey = 0,
  ignoreRequirements,
  onIgnoreRequirements,
  defaultShowPlugins = false,
  onNavigate,
}: Props) {
  const [deploying, setDeploying] = useState(false);
  const [purging, setPurging] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [issue, setIssue] = useState<UserFacingIssue | null>(null);
  const [deployStatus, setDeployStatus] = useState<string | null>(null);
  const [supportsPlugins, setSupportsPlugins] = useState(false);
  const [showPlugins, setShowPlugins] = useState(defaultShowPlugins);
  const [fomodOpen, setFomodOpen] = useState(false);
  const [fomodConfig, setFomodConfig] = useState<FomodConfig | null>(null);
  const [fomodTarget, setFomodTarget] = useState<ModManifest | null>(null);
  const [columns, setColumns] = useState(loadModColumns);
  const [searchOpen, setSearchOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [infoMod, setInfoMod] = useState<ModManifest | null>(null);
  const [modFilter, setModFilter] = useState<ModFilter>("all");
  const [conflictsDismissed, setConflictsDismissed] = useState(false);
  const [checkingUpdates, setCheckingUpdates] = useState(false);
  const [appSettings, setAppSettings] = useState<AppSettings | null>(null);
  const [driftDetected, setDriftDetected] = useState(false);
  const [deployIssues, setDeployIssues] = useState<UserFacingIssue[]>([]);
  const lastSelectedRef = useRef<string | null>(null);

  const { report, loading, blockingIssues, advisoryIssues, ready } =
    usePreflight({
      game,
      mods: queue.mods,
      enabledIds: queue.enabledIds,
      stagingDir: queue.stagingDir,
      profileId: game.profileId,
      deployPathOverride: queue.deployPathOverride,
      conflictResolutions: queue.conflictResolutions,
      enabled: true,
    });

  const conflictModIds = useMemo(
    () => conflictModIdsFromIssues(report.issues),
    [report.issues],
  );

  useEffect(() => {
    void getAppSettings().then(setAppSettings).catch(console.error);
  }, []);

  useEffect(() => {
    setShowPlugins(defaultShowPlugins);
  }, [defaultShowPlugins, game.id]);

  useEffect(() => {
    setConflictsDismissed(false);
  }, [game.id, report.issues.length]);

  const updateCount = useMemo(
    () => queue.mods.filter((m) => m.nexus?.updateAvailable).length,
    [queue.mods],
  );

  const filterCounts = useMemo(
    (): Partial<Record<ModFilter, number>> => ({
      enabled: queue.mods.filter((m) => queue.enabledIds.includes(m.id)).length,
      disabled: queue.mods.filter((m) => !queue.enabledIds.includes(m.id))
        .length,
      conflicts: conflictModIds.size,
      fomod: queue.mods.filter(
        (m) => m.installState === "pendingFomod" || m.needsFomod,
      ).length,
      updates: updateCount,
    }),
    [queue.mods, queue.enabledIds, conflictModIds, updateCount],
  );

  const displayedMods = useMemo(
    () =>
      applyModFilter(queue.mods, modFilter, queue.enabledIds, conflictModIds),
    [queue.mods, modFilter, queue.enabledIds, conflictModIds],
  );

  const conflictIssues = useMemo(
    () =>
      report.issues.filter(
        (i) => i.id.startsWith("conflict-") || i.id.startsWith("missing-dep-"),
      ),
    [report.issues],
  );

  const runUpdateCheck = useCallback(async () => {
    setCheckingUpdates(true);
    try {
      await checkModUpdates(game.id);
      const state = await getGameState(game.id);
      onModsUpdated(state.library.mods.map(libraryModToManifest));
    } catch (e) {
      if (e && typeof e === "object" && "title" in e)
        setIssue(e as UserFacingIssue);
      else console.error(e);
    } finally {
      setCheckingUpdates(false);
    }
  }, [game.id, onModsUpdated]);

  const downloadUpdates = useCallback(async () => {
    setCheckingUpdates(true);
    try {
      for (const mod of queue.mods) {
        if (!mod.nexus?.updateAvailable) continue;
        await enqueueNxmDownload(
          game.id,
          {
            gameDomain: mod.nexus.domain,
            modId: mod.nexus.modId,
            fileId: mod.nexus.fileId,
          },
          mod.name,
        );
      }
    } catch (e) {
      if (e && typeof e === "object" && "title" in e)
        setIssue(e as UserFacingIssue);
      else console.error(e);
    } finally {
      setCheckingUpdates(false);
    }
  }, [game.id, queue.mods]);

  useEffect(() => {
    if (infoMod && !queue.mods.some((m) => m.id === infoMod.id)) {
      setInfoMod(null);
    }
  }, [infoMod, queue.mods]);

  useEffect(() => {
    if (appSettings?.updateCheckMode !== "onStartup") return;
    if (queue.mods.some((m) => m.nexus)) void runUpdateCheck();
  }, [appSettings?.updateCheckMode, game.id]); // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    getGameProfile(game)
      .then((p) => setSupportsPlugins(p.supportsPlugins))
      .catch(console.error);
    getDeployState(game.id)
      .then((response) => {
        if (!response) {
          setDriftDetected(false);
          setDeployIssues([]);
          return;
        }
        const { state } = response;
        const outOfSync =
          !state.report.verified ||
          state.report.missing > 0 ||
          state.report.mismatched > 0;
        setDriftDetected(outOfSync || response.driftDetected);
        setDeployIssues(state.report.issues);
        const when = new Date(
          state.manifest.deployedAt * 1000,
        ).toLocaleString();
        setDeployStatus(
          state.report.verified
            ? `${state.report.linked} files verified · ${when}`
            : `${state.report.linked} linked, ${state.report.missing} missing${state.report.mismatched > 0 ? `, ${state.report.mismatched} mismatched` : ""} · ${when}`,
        );
      })
      .catch(console.error);
  }, [game, deploying, deployRefreshKey]);

  const handleSelect = useCallback(
    (modId: string, additive: boolean, range: boolean) => {
      setSelectedIds((prev) => {
        const next = new Set(additive || range ? prev : []);
        if (range && lastSelectedRef.current) {
          const ids = displayedMods.map((m) => m.id);
          const a = ids.indexOf(lastSelectedRef.current!);
          const b = ids.indexOf(modId);
          if (a >= 0 && b >= 0) {
            const [start, end] = a < b ? [a, b] : [b, a];
            for (let i = start; i <= end; i++) next.add(ids[i]!);
          }
        } else if (additive) {
          if (next.has(modId)) next.delete(modId);
          else next.add(modId);
        } else {
          next.clear();
          next.add(modId);
        }
        lastSelectedRef.current = modId;
        return next;
      });
    },
    [displayedMods],
  );

  const addFiles = async () => {
    const paths = await pickModPaths();
    if (paths.length > 0) {
      const result = await ingestForGame(game.id, paths);
      onIngested(result.mods, result.stagingDir);
      const pending = result.mods.find(
        (m) => m.installState === "pendingFomod" || m.needsFomod,
      );
      if (pending?.slug) void openFomodWizard(pending);
    }
  };

  const openFomodWizard = async (mod: ModManifest) => {
    if (!mod.slug) return;
    try {
      const config = await parseFomod(game.id, mod.slug);
      setFomodTarget(mod);
      setFomodConfig(config);
      setFomodOpen(true);
    } catch (e) {
      console.error(e);
    }
  };

  const completeFomod = async (selections: string[]) => {
    if (!fomodTarget?.slug) return;
    try {
      const updated = await applyFomod(
        game.id,
        fomodTarget.id,
        fomodTarget.slug,
        selections,
      );
      onModsUpdated(queue.mods.map((m) => (m.id === updated.id ? updated : m)));
      onEnableMod(updated.id);
    } catch (e) {
      console.error(e);
    }
  };

  const refresh = async () => {
    setRefreshing(true);
    try {
      const state = await getGameState(game.id);
      onModsUpdated(state.library.mods.map(libraryModToManifest));
      if (appSettings?.updateCheckMode === "onRefresh") {
        await runUpdateCheck();
      }
    } catch (e) {
      console.error(e);
    } finally {
      setRefreshing(false);
    }
  };

  const runDeploy = useCallback(
    async (enabledOverride?: string[], modsOverride?: ModManifest[]) => {
      if (!ready && blockingIssues.length > 0) {
        setIssue(blockingIssues[0] ?? null);
        return;
      }
      const requirementBlockers = report.issues.filter((i) =>
        i.id.startsWith("req-"),
      );
      if (!ignoreRequirements && requirementBlockers.length > 0) {
        setIssue(requirementBlockers[0] ?? null);
        return;
      }
      if (!queue.stagingDir) {
        setIssue({
          id: "no-staging",
          title: "Staging not ready",
          explanation: "Add a mod to your library first.",
          impact: "Import mod files to continue.",
          choices: [],
        });
        return;
      }
      const enabledIds = enabledOverride ?? queue.enabledIds;
      const mods = modsOverride ?? queue.mods;
      const pendingFomod = mods.some(
        (m) =>
          enabledIds.includes(m.id) &&
          (m.installState === "pendingFomod" || m.needsFomod),
      );
      if (pendingFomod) {
        setIssue({
          id: "pending-fomod",
          title: "FOMOD setup required",
          explanation:
            "One or more enabled mods need installer options before deploy.",
          impact: "Configure FOMOD options for pending mods.",
          choices: [],
        });
        return;
      }

      setDeploying(true);
      try {
        const partition = await checkPartition(
          queue.stagingDir,
          game.installPath,
        );
        if (!partition.samePartition && partition.guidance) {
          setIssue(partition.guidance);
          return;
        }
        const result = await deployGameMods({
          gameId: game.id,
          gameDir: game.installPath,
          profileId: game.profileId,
          stagingDir: queue.stagingDir,
          mods,
          enabledIds,
          conflictResolutions: queue.conflictResolutions,
          ignoreRequirements,
          deployPathOverride: queue.deployPathOverride,
        });
        onDeployComplete();
        setDeployStatus(result.summary);
        if (!appSettings?.autoDeployOnChange) {
          setIssue({
            id: "deploy-success",
            title: result.report.verified
              ? "Deploy verified"
              : "Deploy completed with warnings",
            explanation: result.summary,
            impact: result.report.verified
              ? "Your enabled mods are linked into the game folder."
              : "Some files may need attention.",
            choices: [
              {
                id: "acknowledge",
                label: "Done",
                description: "Close",
                recommended: true,
              },
            ],
          });
        }
      } catch (e) {
        if (e && typeof e === "object" && "title" in e)
          setIssue(e as UserFacingIssue);
      } finally {
        setDeploying(false);
      }
    },
    [
      appSettings,
      blockingIssues,
      game.id,
      game.installPath,
      game.profileId,
      ignoreRequirements,
      onDeployComplete,
      queue.conflictResolutions,
      queue.deployPathOverride,
      queue.enabledIds,
      queue.mods,
      queue.stagingDir,
      ready,
      report.issues,
    ],
  );

  useEffect(() => {
    if (!isTauri()) return;
    const unsubs: Array<() => void> = [];
    listen<{ gameId: string }>("download://completed", (event) => {
      if (event.payload.gameId !== game.id) return;
      void getAppSettings().then((raw) => {
        if (mergeSettings(raw).autoDeployOnChange) {
          void runDeploy();
        }
      });
    }).then((un) => unsubs.push(un));
    listen<{ gameId: string }>("deploy://started", (event) => {
      if (event.payload.gameId === game.id) setDeploying(true);
    }).then((un) => unsubs.push(un));
    listen<{ gameId: string; summary?: string }>(
      "deploy://completed",
      (event) => {
        if (event.payload.gameId !== game.id) return;
        if (event.payload.summary) setDeployStatus(event.payload.summary);
        onDeployComplete();
      },
    ).then((un) => unsubs.push(un));
    return () => unsubs.forEach((un) => un());
  }, [game.id, onDeployComplete, runDeploy]);

  const handleReorder = async (modIds: string[]) => {
    const byId = new Map(queue.mods.map((m) => [m.id, m]));
    const reordered = modIds
      .map((id) => byId.get(id))
      .filter((m): m is ModManifest => m !== undefined);
    for (const mod of queue.mods) {
      if (!modIds.includes(mod.id)) reordered.push(mod);
    }
    onModsReordered?.(reordered);

    try {
      const library = await reorderLibraryMods(game.id, modIds);
      const mods = library.mods.map(libraryModToManifest);
      onModsUpdated(mods);
      onModsReordered?.(mods);
      if (appSettings?.autoDeployOnChange) {
        await runDeploy(queue.enabledIds, mods);
      }
    } catch (e) {
      console.error(e);
      onModsUpdated(queue.mods);
      onModsReordered?.(queue.mods);
    }
  };

  const deploy = async () => {
    if (
      appSettings?.confirmBeforeDeploy &&
      !window.confirm(
        "Deploy enabled mods into your game folder? This updates hardlinks/links for the selected loadout.",
      )
    ) {
      return;
    }
    await runDeploy();
  };

  const purge = async () => {
    setPurging(true);
    try {
      const result = await purgeDeployedMods(game.id);
      setDeployStatus(
        result.removedFiles > 0
          ? `Purged ${result.removedFiles} file(s) from game folder`
          : "Nothing to purge",
      );
    } catch (e) {
      if (e && typeof e === "object" && "title" in e)
        setIssue(e as UserFacingIssue);
    } finally {
      setPurging(false);
    }
  };

  const handleIssueChoice = useCallback(
    (choiceId: string) => {
      if (!issue) return;
      if (issue.id.startsWith("conflict-")) {
        onConflictChoice(issue.id, choiceId);
        setIssue(null);
        return;
      }
      if (choiceId === "fix-bsa-timestamps") {
        void fixBsaTimestamps(game.installPath)
          .then((count) => {
            setDeployStatus(
              `Updated timestamps on ${count} vanilla archive(s).`,
            );
            setIssue(null);
          })
          .catch((e) => {
            if (e && typeof e === "object" && "title" in e)
              setIssue(e as UserFacingIssue);
          });
        return;
      }
      if (choiceId === "open-game-folder") {
        void openPathSafe(game.installPath).then((err) => {
          if (err) {
            setIssue({
              id: "open-folder-failed",
              title: "Could not open game folder",
              explanation: err,
              impact:
                "Check that the game path still exists in Settings → Games.",
              choices: [],
            });
            return;
          }
          setIssue(null);
        });
        return;
      }
      if (choiceId === "open-settings") {
        onNavigate?.("settings");
        setIssue(null);
        return;
      }
      if (choiceId === "reinstall") {
        setIssue(null);
        void runDeploy();
        return;
      }
      if (choiceId === "continue-anyway") {
        onIgnoreRequirements();
        setIssue(null);
        return;
      }
      if (choiceId.startsWith("enable-")) {
        onEnableMod(choiceId.replace(/^enable-/, ""));
        setIssue(null);
        return;
      }
      setIssue(null);
    },
    [
      game.installPath,
      issue,
      onConflictChoice,
      onEnableMod,
      onIgnoreRequirements,
      onNavigate,
      runDeploy,
    ],
  );

  const handleToggleColumn = (id: string) => {
    setColumns((prev) => {
      const next = toggleColumn(prev, id);
      saveModColumns(next);
      return next;
    });
  };

  const handleReinstall = async (mod: ModManifest) => {
    try {
      const updated = await reinstallMod(game.id, mod.id);
      onModsUpdated(queue.mods.map((m) => (m.id === updated.id ? updated : m)));
      if (updated.installState === "pendingFomod" || updated.needsFomod) {
        void openFomodWizard(updated);
      }
    } catch (e) {
      if (e && typeof e === "object" && "title" in e)
        setIssue(e as UserFacingIssue);
      else console.error(e);
    }
  };

  const handleOpenFolder = (mod: ModManifest) => {
    void (async () => {
      if (!mod.slug?.trim()) {
        setIssue({
          id: "open-mod-folder-failed",
          title: "Mod folder unavailable",
          explanation:
            "This mod has no staging folder name yet. Try reinstalling or refreshing the mod list.",
          impact:
            "FOMOD mods must finish installation before their folder can be opened.",
          choices: [],
        });
        return;
      }

      const stagingRoot = queue.stagingDir?.trim();
      if (!stagingRoot) {
        setIssue({
          id: "open-mod-folder-failed",
          title: "Staging folder unknown",
          explanation:
            "Supervisor has not loaded a staging path for this game yet.",
          impact: "Switch away from Mods and back, or restart the app.",
          choices: [],
        });
        return;
      }

      const separator = stagingRoot.includes("\\") ? "\\" : "/";
      const slugPath = `${stagingRoot}${separator}${mod.slug}`;
      const err = await openPathSafe(slugPath);
      if (err) {
        setIssue({
          id: "open-mod-folder-failed",
          title: "Mod folder not found",
          explanation: err,
          impact:
            "The mod may have been removed from staging. Reinstall the archive or purge and redeploy.",
          choices: [
            {
              id: "reinstall",
              label: "Redeploy mods",
              description: "Sync the game folder with your current loadout.",
              recommended: true,
            },
          ],
        });
      }
    })();
  };

  const handleRemoveMod = (modId: string) => {
    setInfoMod((prev) => (prev?.id === modId ? null : prev));
    onRemove(modId);
  };

  const handleToggleMod = (modId: string) => {
    const nextEnabled = queue.enabledIds.includes(modId)
      ? queue.enabledIds.filter((id) => id !== modId)
      : [...queue.enabledIds, modId];
    onToggle(modId);
    if (appSettings?.autoDeployOnChange) {
      void runDeploy(nextEnabled);
    }
  };

  const handleLoadoutChange = (loadout: Loadout) => {
    onLoadoutChange(loadout);
    if (appSettings?.autoDeployOnChange) {
      void runDeploy(loadout.enabledModIds);
    }
  };

  const handleModUpdated = (updated: ModManifest) => {
    onModsUpdated(queue.mods.map((m) => (m.id === updated.id ? updated : m)));
    setInfoMod((prev) => (prev?.id === updated.id ? updated : prev));
  };

  return (
    <div className="flex h-full min-h-0 flex-col bg-content-panel">
      <ModToolbar
        game={game}
        gameId={game.id}
        loadoutId={queue.activeLoadoutId}
        loadoutName={queue.loadoutName}
        deployPathOverride={queue.deployPathOverride}
        driftDetected={driftDetected}
        onDeployPathOverride={onDeployPathOverride}
        onPurgeComplete={onDeployComplete}
        onDriftChecked={setDriftDetected}
        onLoadoutChange={handleLoadoutChange}
        onAdd={() => void addFiles()}
        onRefresh={() => void refresh()}
        onDeploy={() => void deploy()}
        onPurge={() => void purge()}
        onResetManifest={() => void purge().then(() => void refresh())}
        deploying={deploying}
        purging={purging}
        refreshing={refreshing}
        searchOpen={searchOpen}
        onSearchToggle={() => setSearchOpen((v) => !v)}
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
        columns={columns}
        onToggleColumn={handleToggleColumn}
        onShowCategories={() => setSearchOpen(true)}
        onManageRules={() => {
          setModFilter("conflicts");
          setConflictsDismissed(false);
        }}
        onOpenStaging={() =>
          void openStagingFolder(game.id).catch(console.error)
        }
        onOpenGameFolder={() =>
          void openPath(game.installPath).catch(console.error)
        }
        onShowHistory={() =>
          setIssue(
            deployStatus
              ? {
                  id: "deploy-history",
                  title: "Deploy history",
                  explanation: deployStatus,
                  impact: "Your most recent deploy for this game.",
                  choices: [
                    {
                      id: "acknowledge",
                      label: "Close",
                      description: "",
                      recommended: true,
                    },
                  ],
                }
              : {
                  id: "deploy-history-empty",
                  title: "No deploy history",
                  explanation: "You have not deployed mods for this game yet.",
                  impact: "Deploy mods to link them into your game folder.",
                  choices: [
                    {
                      id: "acknowledge",
                      label: "Close",
                      description: "",
                      recommended: true,
                    },
                  ],
                },
          )
        }
        onCheckUpdates={() => void runUpdateCheck()}
        onDownloadUpdates={() => void downloadUpdates()}
        checkingUpdates={checkingUpdates}
        updateCount={updateCount}
      />

      <ModFilterBar
        active={modFilter}
        counts={filterCounts}
        onChange={setModFilter}
      />

      {!conflictsDismissed && conflictIssues.length > 0 && (
        <ConflictPanel
          issues={conflictIssues}
          onResolve={(issueId, choiceId) => {
            if (choiceId === "cancel") return;
            if (issueId.startsWith("conflict-")) {
              onConflictChoice(issueId, choiceId);
              return;
            }
            if (choiceId.startsWith("enable-")) {
              onEnableMod(choiceId.replace(/^enable-/, ""));
            }
          }}
          onDismiss={() => setConflictsDismissed(true)}
        />
      )}

      {!loading && !ready && blockingIssues.length > 0 && (
        <div className="shrink-0 border-b border-border bg-warning/10 px-3 py-1.5 text-xs text-warning">
          {report.summary}{" "}
          <button
            type="button"
            className="underline"
            onClick={() => setIssue(blockingIssues[0] ?? null)}
          >
            Review issues
          </button>
        </div>
      )}

      {!loading && advisoryIssues.length > 0 && (
        <DeployAdvisoryBanner
          issues={advisoryIssues}
          onReview={(issue) => setIssue(issue)}
        />
      )}

      {deployIssues.length > 0 && (
        <DeployAdvisoryBanner
          issues={deployIssues}
          onReview={(issue) => setIssue(issue)}
        />
      )}

      {driftDetected && deployIssues.length === 0 && (
        <div className="shrink-0 border-b border-border px-2 py-1.5 text-sm text-warning">
          <p className="font-medium">Deploy drift detected</p>
          <p className="mt-0.5 text-text-secondary">
            Files in your game folder no longer match the last deploy. Purge and
            redeploy from the menu (⋮), or click Deploy.
          </p>
        </div>
      )}

      <ModBatchBar
        count={selectedIds.size}
        onEnableAll={() => {
          const nextEnabled = [
            ...queue.enabledIds,
            ...[...selectedIds].filter((id) => !queue.enabledIds.includes(id)),
          ];
          selectedIds.forEach((id) => {
            if (!queue.enabledIds.includes(id)) onToggle(id);
          });
          if (appSettings?.autoDeployOnChange) void runDeploy(nextEnabled);
        }}
        onDisableAll={() => {
          const nextEnabled = queue.enabledIds.filter(
            (id) => !selectedIds.has(id),
          );
          selectedIds.forEach((id) => {
            if (queue.enabledIds.includes(id)) onToggle(id);
          });
          if (appSettings?.autoDeployOnChange) void runDeploy(nextEnabled);
        }}
        onRemoveAll={() => {
          selectedIds.forEach((id) => handleRemoveMod(id));
          setSelectedIds(new Set());
        }}
        onClear={() => setSelectedIds(new Set())}
      />

      <div className="relative min-h-0 flex-1 flex flex-col overflow-hidden">
        {showPlugins && supportsPlugins ? (
          <div className="min-h-0 flex-1 overflow-y-auto p-4">
            <PluginList
              game={game}
              mods={queue.mods}
              enabledIds={queue.enabledIds}
            />
          </div>
        ) : (
          <ModTable
            mods={displayedMods}
            libraryModCount={queue.mods.length}
            viewFilter={modFilter}
            libraryMods={queue.mods}
            enabledIds={queue.enabledIds}
            columns={columns}
            selectedIds={selectedIds}
            conflictModIds={conflictModIds}
            onSelect={handleSelect}
            onToggle={handleToggleMod}
            onRemove={handleRemoveMod}
            onConfigureFomod={(m) => void openFomodWizard(m)}
            onShowInfo={setInfoMod}
            onReinstall={(m) => void handleReinstall(m)}
            onOpenFolder={handleOpenFolder}
            onReorder={(ids) => void handleReorder(ids)}
            globalFilter={searchQuery}
            onClearGlobalFilter={() => setSearchQuery("")}
            onClearViewFilter={() => setModFilter("all")}
          />
        )}
      </div>

      <ModInfoPanel
        mod={infoMod}
        gameId={game.id}
        gameDomain={game.nexusDomain}
        onClose={() => setInfoMod(null)}
        onOpenFolder={handleOpenFolder}
        onModUpdated={handleModUpdated}
      />

      {deployStatus && (
        <div className="shrink-0 border-t border-border px-3 py-1 text-xs text-text-muted">
          Last deploy: {deployStatus}
        </div>
      )}

      <ModDropzone gameId={game.id} onIngested={onIngested} compact />

      {fomodConfig && fomodTarget && (
        <FomodWizard
          open={fomodOpen}
          onOpenChange={setFomodOpen}
          config={fomodConfig}
          modName={fomodTarget.name}
          onComplete={(selections) => void completeFomod(selections)}
          onInstallDefaults={() => {
            const defaults = fomodConfig.steps.flatMap((s) => {
              const def = s.options.find((o) => o.isDefault) ?? s.options[0];
              return def ? [def.id] : [];
            });
            void completeFomod(defaults);
            setFomodOpen(false);
          }}
        />
      )}

      <IssueModal
        issue={issue}
        onClose={() => setIssue(null)}
        onChoice={handleIssueChoice}
      />
    </div>
  );
}
