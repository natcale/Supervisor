/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { scanGames } from "@/lib/api/games";
import {
  emptyQueue,
  gameMatchesNxm,
  type DetectedGame,
  type GameQueue,
  type GameQueues,
  type Loadout,
  type ModManifest,
  type NxmPayload,
  type ShellView,
  type DownloadJob,
} from "@/types";
import {
  enqueueNxmDownload,
  getGameState,
  ingestedToManifest,
  libraryModToManifest,
  removeLibraryMod,
  updateLoadout,
  getAppSettings,
  updateAppSettings,
} from "@/lib/tauri";
import { mergeSettings } from "@/lib/settings-defaults";
import { isTauri } from "@/lib/env";
import { listenSettingsChanged } from "@/lib/settings-events";

function stateToQueue(loadout: Loadout, mods: ModManifest[], stagingDir: string): GameQueue {
  return {
    mods,
    enabledIds: loadout.enabledModIds,
    stagingDir,
    conflictResolutions: loadout.conflictResolutions,
    deployPathOverride: loadout.deployPathOverride,
    activeLoadoutId: loadout.id,
    loadoutName: loadout.name,
  };
}

export function useGameState() {
  const [shellView, setShellView] = useState<ShellView>("home");
  const [selectedGame, setSelectedGame] = useState<DetectedGame | null>(null);
  const [games, setGames] = useState<DetectedGame[]>([]);
  const [gamesLoading, setGamesLoading] = useState(true);
  const [queues, setQueues] = useState<GameQueues>({});
  const [deployRefreshKey, setDeployRefreshKey] = useState(0);
  const loadoutsRef = useRef<Record<string, Loadout>>({});
  const scanned = useRef(false);
  const [nxmNotice, setNxmNotice] = useState<string | null>(null);
  const [nxmStatus, setNxmStatus] = useState<string | null>(null);

  const currentQueue = useMemo(
    () => (selectedGame ? queues[selectedGame.id] ?? emptyQueue() : emptyQueue()),
    [queues, selectedGame],
  );

  const persistLoadout = useCallback(async (gameId: string, queue: GameQueue) => {
    const base = loadoutsRef.current[gameId];
    if (!base) return;
    const updated: Loadout = {
      ...base,
      enabledModIds: queue.enabledIds,
      conflictResolutions: queue.conflictResolutions,
      deployPathOverride: queue.deployPathOverride,
    };
    try {
      const saved = await updateLoadout(gameId, updated);
      loadoutsRef.current[gameId] = saved;
    } catch (e) {
      console.error("Failed to save loadout:", e);
    }
  }, []);

  const stripNxPlaceholder = useCallback(
    (gameId: string, placeholderId: string) => {
      setQueues((prev) => {
        const q = prev[gameId];
        if (!q || !q.mods.some((m) => m.id === placeholderId)) return prev;
        const next: GameQueue = {
          ...q,
          mods: q.mods.filter((m) => m.id !== placeholderId),
          enabledIds: q.enabledIds.filter((id) => id !== placeholderId),
        };
        void persistLoadout(gameId, next);
        return { ...prev, [gameId]: next };
      });
    },
    [persistLoadout],
  );

  useEffect(() => {
    if (!isTauri()) return;
    const unsubs: Array<() => void> = [];
    listen<DownloadJob>("download://updated", (event) => {
      if (event.payload.status !== "failed") return;
      const placeholderId = `nxm-${event.payload.modId}-${event.payload.fileId}`;
      stripNxPlaceholder(event.payload.gameId, placeholderId);
    }).then((un) => unsubs.push(un));
    return () => unsubs.forEach((un) => un());
  }, [stripNxPlaceholder]);

  const persistRememberedGame = useCallback(async (gameId: string) => {
    try {
      const s = mergeSettings(await getAppSettings());
      if (!s.rememberLastGame) return;
      await updateAppSettings(mergeSettings({ ...s, lastGameId: gameId }));
    } catch (e) {
      console.error(e);
    }
  }, []);

  const applyQueue = useCallback(
    (gameId: string, queue: GameQueue, persist = true) => {
      setQueues((prev) => ({ ...prev, [gameId]: queue }));
      if (persist) void persistLoadout(gameId, queue);
    },
    [persistLoadout],
  );

  const selectGame = useCallback(
    async (game: DetectedGame) => {
      setSelectedGame(game);
      try {
        const state = await getGameState(game.id);
        loadoutsRef.current[game.id] = state.loadout;
        const mods = state.library.mods.map(libraryModToManifest);
        applyQueue(game.id, stateToQueue(state.loadout, mods, state.stagingDir), false);
      } catch {
        setQueues((prev) => {
          if (prev[game.id]) return prev;
          return { ...prev, [game.id]: emptyQueue() };
        });
      } finally {
        void persistRememberedGame(game.id);
      }
    },
    [applyQueue, persistRememberedGame],
  );

  useEffect(() => {
    if (!isTauri()) {
      setGamesLoading(false);
      return;
    }
    if (scanned.current) return;
    scanned.current = true;
    void scanGames()
      .then(async (result) => {
        setGames(result.games);
        if (result.games.length === 0) return;
        let pick = result.games[0]!;
        try {
          const gs = mergeSettings(await getAppSettings());
          if (gs.rememberLastGame && gs.lastGameId) {
            const remembered = result.games.find((g) => g.id === gs.lastGameId);
            if (remembered) pick = remembered;
          }
        } catch (e) {
          console.error(e);
        }
        void selectGame(pick);
      })
      .catch(console.error)
      .finally(() => setGamesLoading(false));
  }, [selectGame]);

  useEffect(() => {
    if (!isTauri()) return;

    const scanConfigKey = (settings: ReturnType<typeof mergeSettings>) =>
      [
        settings.scanSteam,
        settings.scanEpic,
        settings.scanGog,
        settings.scanHeroic,
        settings.showUnmoddableGames,
      ].join(":");

    const scanConfigRef = { current: "" };

    let unlisten: (() => void) | undefined;
    void getAppSettings()
      .then((settings) => {
        scanConfigRef.current = scanConfigKey(mergeSettings(settings));
      })
      .catch(console.error);

    void listenSettingsChanged((settings) => {
      const next = mergeSettings(settings);
      const key = scanConfigKey(next);
      if (key === scanConfigRef.current) return;
      scanConfigRef.current = key;

      void scanGames({ includeAll: next.showUnmoddableGames })
        .then((result) => {
          setGames(result.games);
          if (result.games.length === 0) {
            setSelectedGame(null);
            return;
          }
          const currentId = selectedGame?.id;
          const stillThere = currentId
            ? result.games.find((game) => game.id === currentId)
            : undefined;
          if (stillThere) return;
          let pick = result.games[0]!;
          if (next.rememberLastGame && next.lastGameId) {
            const remembered = result.games.find((game) => game.id === next.lastGameId);
            if (remembered) pick = remembered;
          }
          void selectGame(pick);
        })
        .catch(console.error);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => unlisten?.();
  }, [selectGame, selectedGame?.id]);

  const addMods = useCallback(
    (gameId: string, mods: ModManifest[], stagingDir: string, switchToMods = true) => {
      setQueues((prev) => {
        const q = prev[gameId] ?? emptyQueue();
        const existing = new Set(q.mods.map((m) => m.id));
        const merged = [...q.mods];
        for (const mod of mods) {
          const idx = merged.findIndex((m) => m.id === mod.id);
          if (idx >= 0) merged[idx] = mod;
          else if (!existing.has(mod.id)) merged.push(mod);
        }
        const enabled = new Set(q.enabledIds);
        for (const mod of mods) {
          if (mod.installState !== "pendingFomod") enabled.add(mod.id);
        }
        const next = {
          ...q,
          mods: merged,
          enabledIds: Array.from(enabled),
          stagingDir: stagingDir || q.stagingDir,
        };
        void persistLoadout(gameId, next);
        return { ...prev, [gameId]: next };
      });
      if (switchToMods) setShellView("mods");
    },
    [persistLoadout],
  );

  const toggleMod = useCallback(
    (gameId: string, modId: string) => {
      setQueues((prev) => {
        const q = prev[gameId] ?? emptyQueue();
        const enabledIds = q.enabledIds.includes(modId)
          ? q.enabledIds.filter((id) => id !== modId)
          : [...q.enabledIds, modId];
        const next = { ...q, enabledIds };
        void persistLoadout(gameId, next);
        return { ...prev, [gameId]: next };
      });
    },
    [persistLoadout],
  );

  const removeMod = useCallback(
    (gameId: string, modId: string) => {
      setQueues((prev) => {
        const q = prev[gameId] ?? emptyQueue();
        const next = {
          ...q,
          mods: q.mods.filter((m) => m.id !== modId),
          enabledIds: q.enabledIds.filter((id) => id !== modId),
        };
        void persistLoadout(gameId, next);
        return { ...prev, [gameId]: next };
      });
      void removeLibraryMod(gameId, modId)
        .then((library) => {
          setQueues((prev) => {
            const q = prev[gameId] ?? emptyQueue();
            const mods = library.mods.map(libraryModToManifest);
            const modIds = new Set(mods.map((m) => m.id));
            return {
              ...prev,
              [gameId]: {
                ...q,
                mods,
                enabledIds: q.enabledIds.filter((id) => modIds.has(id)),
              },
            };
          });
          setDeployRefreshKey((k) => k + 1);
        })
        .catch(console.error);
    },
    [persistLoadout],
  );

  const reorderMods = useCallback((gameId: string, mods: ModManifest[]) => {
    setQueues((prev) => {
      const q = prev[gameId] ?? emptyQueue();
      return { ...prev, [gameId]: { ...q, mods } };
    });
  }, []);

  const setConflictWinner = useCallback(
    (gameId: string, path: string, modId: string) => {
      setQueues((prev) => {
        const q = prev[gameId] ?? emptyQueue();
        const next = {
          ...q,
          conflictResolutions: { ...q.conflictResolutions, [path]: modId },
        };
        void persistLoadout(gameId, next);
        return { ...prev, [gameId]: next };
      });
    },
    [persistLoadout],
  );

  const enableMod = useCallback(
    (gameId: string, modId: string) => {
      setQueues((prev) => {
        const q = prev[gameId] ?? emptyQueue();
        if (q.enabledIds.includes(modId)) return prev;
        const next = { ...q, enabledIds: [...q.enabledIds, modId] };
        void persistLoadout(gameId, next);
        return { ...prev, [gameId]: next };
      });
    },
    [persistLoadout],
  );

  const setDeployPath = useCallback(
    (gameId: string, path: string | undefined) => {
      setQueues((prev) => {
        const q = prev[gameId] ?? emptyQueue();
        let next: GameQueue;
        if (!path) {
          const rest = { ...q };
          delete rest.deployPathOverride;
          next = rest;
        } else {
          next = { ...q, deployPathOverride: path };
        }
        void persistLoadout(gameId, next);
        return { ...prev, [gameId]: next };
      });
    },
    [persistLoadout],
  );

  const handleLoadoutChange = useCallback((gameId: string, loadout: Loadout) => {
    loadoutsRef.current[gameId] = loadout;
    setQueues((prev) => {
      const q = prev[gameId] ?? emptyQueue();
      return {
        ...prev,
        [gameId]: {
          ...q,
          enabledIds: loadout.enabledModIds,
          conflictResolutions: loadout.conflictResolutions,
          deployPathOverride: loadout.deployPathOverride,
          activeLoadoutId: loadout.id,
          loadoutName: loadout.name,
        },
      };
    });
  }, []);

  const handleIngested = useCallback(
    (mods: ModManifest[], stagingDir: string) => {
      if (!selectedGame) return;
      addMods(selectedGame.id, mods, stagingDir, true);
    },
    [addMods, selectedGame],
  );

  const handleModLink = useCallback(
    async (payload: Extract<NxmPayload, { kind: "modDownload" }>) => {
      const match = games.find((g) => gameMatchesNxm(g, payload.gameDomain));
      const game = match ?? selectedGame;
      if (!game) {
        setNxmNotice(
          `No installed game matches Nexus domain "${payload.gameDomain}". Add the game in Settings → Games, then try again.`,
        );
        setShellView("settings");
        return;
      }

      setNxmNotice(null);
      setNxmStatus(
        `Nexus Mods sent a download for ${payload.gameDomain} · Mod #${payload.modId} · File #${payload.fileId}`,
      );

      if (!selectedGame || selectedGame.id !== game.id) {
        await selectGame(game);
      }

      setShellView("downloads");

      const id = `nxm-${payload.modId}-${payload.fileId}`;
      const modName = `${payload.gameDomain} mod #${payload.modId}`;
      const placeholder: ModManifest = {
        id,
        name: modName,
        files: [],
        dependencies: [],
        installState: "pendingFomod",
      };
      addMods(game.id, [placeholder], queues[game.id]?.stagingDir ?? "", false);

      try {
        await enqueueNxmDownload(
          game.id,
          {
            gameDomain: payload.gameDomain,
            modId: payload.modId,
            fileId: payload.fileId,
            key: payload.key,
            expires: payload.expires,
            userId: payload.userId,
          },
          modName,
        );
      } catch (e) {
        stripNxPlaceholder(game.id, id);
        if (e && typeof e === "object" && "title" in e) {
          console.error((e as { title: string }).title);
        } else {
          console.error("NXM enqueue failed:", e);
        }
      }
    },
    [addMods, games, queues, selectedGame, selectGame, stripNxPlaceholder],
  );

  const handleDownloadComplete = useCallback(
    (gameId: string, mods: ModManifest[], stagingDir: string) => {
      addMods(gameId, mods.map(ingestedToManifest), stagingDir, true);
      if (selectedGame?.id === gameId) {
        void selectGame(selectedGame);
      }
    },
    [addMods, selectGame, selectedGame],
  );

  const handleGamesLoaded = useCallback((loaded: DetectedGame[]) => {
    setGames(loaded);
    setGamesLoading(false);
  }, []);

  const bumpDeployRefresh = useCallback(() => {
    setDeployRefreshKey((k) => k + 1);
  }, []);

  return {
    shellView,
    setShellView,
    selectedGame,
    games,
    gamesLoading,
    currentQueue,
    deployRefreshKey,
    selectGame,
    toggleMod,
    removeMod,
    reorderMods,
    setConflictWinner,
    enableMod,
    setDeployPath,
    handleLoadoutChange,
    handleIngested,
    handleModLink,
    handleDownloadComplete,
    handleGamesLoaded,
    bumpDeployRefresh,
    nxmNotice,
    clearNxmNotice: () => setNxmNotice(null),
    nxmStatus,
    clearNxmStatus: () => setNxmStatus(null),
  };
}
