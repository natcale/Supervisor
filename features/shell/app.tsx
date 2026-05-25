/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { Titlebar } from "@/features/layout/ui/titlebar";
import { NavSidebar } from "@/features/layout/ui/nav-sidebar";
import { CompactGameSidebar } from "@/features/layout/ui/compact-game-sidebar";
import { MainContent } from "@/features/shell/main-content";
import { AppUpdateBanner } from "@/features/shell/ui/app-update-banner";
import { StatusBar } from "@/features/shell/status-bar";
import { useGameState } from "@/features/shell/model/use-game-state";
import { DownloadQueueProvider, useDownloadQueue } from "@/features/downloads/model/use-download-queue";
import { useThemeLayout } from "@/features/themes/theme-provider";
import { useEffect, useMemo, useState } from "react";
import { getAppSettings } from "@/lib/api/settings";
import { getGameProfile } from "@/lib/api/games";
import { mergeSettings, SETTINGS_DEFAULTS } from "@/lib/settings-defaults";
import { listenSettingsChanged } from "@/lib/settings-events";
import { isTauri } from "@/lib/env";
import { listen } from "@tauri-apps/api/event";
import type { AppSettings, NxmPayload } from "@/types";

export function App() {
  const gameState = useGameState();

  return (
    <DownloadQueueProvider onDownloadComplete={gameState.handleDownloadComplete}>
      <AppShell {...gameState} />
    </DownloadQueueProvider>
  );
}

function AppShell({
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
  handleGamesLoaded,
  bumpDeployRefresh,
  nxmNotice,
  clearNxmNotice,
  nxmStatus,
  clearNxmStatus,
}: ReturnType<typeof useGameState>) {
  const { activeCount } = useDownloadQueue();
  const [alwaysShowPlugins, setAlwaysShowPlugins] = useState(false);
  const [supportsPlugins, setSupportsPlugins] = useState(false);
  const [appSettings, setAppSettings] = useState<AppSettings | null>(null);
  const { getSlot } = useThemeLayout();

  const showCompactBar = useMemo(() => {
    const settings = appSettings ?? SETTINGS_DEFAULTS;
    const themeEnabled = getSlot("shell.compactBar")?.enabled === true;
    if (settings.compactGameSidebarHidden) return false;
    return settings.compactGameSidebar || themeEnabled;
  }, [appSettings, getSlot]);

  useEffect(() => {
    if (!isTauri()) return;
    void getAppSettings().then((s) => {
      const merged = mergeSettings(s);
      setAlwaysShowPlugins(merged.alwaysShowPlugins);
      setAppSettings(merged);
    });

    let unlisten: (() => void) | undefined;
    void listenSettingsChanged((settings) => {
      const merged = mergeSettings(settings);
      setAlwaysShowPlugins(merged.alwaysShowPlugins);
      setAppSettings(merged);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => unlisten?.();
  }, []);

  useEffect(() => {
    if (!selectedGame) {
      setSupportsPlugins(false);
      return;
    }
    void getGameProfile(selectedGame)
      .then((p) => setSupportsPlugins(p.supportsPlugins))
      .catch(() => setSupportsPlugins(false));
  }, [selectedGame]);

  useEffect(() => {
    if (!isTauri()) return;
    let unlisten: (() => void) | undefined;
    listen<NxmPayload>("nxm://received", (event) => {
      if (event.payload.kind === "modDownload") {
        handleModLink(event.payload);
      }
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, [handleModLink]);

  return (
    <div className="relative flex h-screen flex-col overflow-hidden bg-background">
      <Titlebar onNavigate={setShellView} />
      <AppUpdateBanner />

      <main className="flex min-h-0 flex-1 overflow-hidden">
        <NavSidebar
          active={shellView}
          onNavigate={setShellView}
          game={selectedGame}
          modCount={currentQueue.mods.length}
          downloadActive={activeCount}
          supportsPlugins={supportsPlugins}
          alwaysShowPlugins={alwaysShowPlugins}
        />

        <div className="min-h-0 min-w-0 flex-1 overflow-hidden rounded-xl bg-content-panel">
          <MainContent
            view={shellView}
            game={selectedGame}
            games={games}
            gamesLoading={gamesLoading}
            queue={currentQueue}
            deployRefreshKey={deployRefreshKey}
            onSelectGame={(g) => void selectGame(g)}
            onToggle={(modId) => selectedGame && toggleMod(selectedGame.id, modId)}
            onRemove={(modId) => selectedGame && removeMod(selectedGame.id, modId)}
            onIngested={handleIngested}
            onConflictChoice={(path, modId) =>
              selectedGame && setConflictWinner(selectedGame.id, path, modId)
            }
            onEnableMod={(modId) => selectedGame && enableMod(selectedGame.id, modId)}
            onDeployPathOverride={(path) =>
              selectedGame && setDeployPath(selectedGame.id, path)
            }
            onLoadoutChange={(loadout) =>
              selectedGame && handleLoadoutChange(selectedGame.id, loadout)
            }
            onModsUpdated={(mods) => {
              if (!selectedGame) return;
              reorderMods(selectedGame.id, mods);
            }}
            onModsReordered={(mods) => {
              if (selectedGame) reorderMods(selectedGame.id, mods);
            }}
            onDeployComplete={bumpDeployRefresh}
            onGamesLoaded={handleGamesLoaded}
            onNavigate={setShellView}
            nxmNotice={nxmNotice}
            onClearNxmNotice={clearNxmNotice}
          />
        </div>

        {showCompactBar && (
          <CompactGameSidebar
            games={games}
            selectedGame={selectedGame}
            onSelectGame={(g) => void selectGame(g)}
            onNavigate={setShellView}
          />
        )}
      </main>

      <StatusBar
        shellView={shellView}
        game={selectedGame}
        queue={currentQueue}
        deployRefreshKey={deployRefreshKey}
        onNavigate={setShellView}
        nxmStatus={nxmStatus}
        onClearNxmStatus={clearNxmStatus}
      />
    </div>
  );
}
