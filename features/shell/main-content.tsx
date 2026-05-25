/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useEffect, useRef, useState } from "react";
import { getDeployState } from "@/lib/api/deploy";
import { IssueModal } from "@/components/ui/issue-modal";
import { DownloadsPanel } from "@/features/downloads/ui/downloads-panel";
import { WelcomeScreen } from "@/features/home/ui/welcome-screen";
import { ModsTab } from "@/features/mod-manager/ui/mods-tab";
import { SettingsPage } from "@/features/settings/settings-page";
import {
  CollectionsPage,
  LibraryPage,
} from "@/features/shell/placeholder-pages";
import { ingestForGame, useWindowDrop } from "@/lib/drop";
import type {
  DetectedGame,
  GameQueue,
  Loadout,
  ModManifest,
  ShellView,
  UserFacingIssue,
} from "@/types";

type Props = {
  view: ShellView;
  game: DetectedGame | null;
  games: DetectedGame[];
  gamesLoading?: boolean;
  queue: GameQueue;
  deployRefreshKey: number;
  onSelectGame: (game: DetectedGame) => void;
  onToggle: (modId: string) => void;
  onRemove: (modId: string) => void;
  onIngested: (mods: ModManifest[], stagingDir: string) => void;
  onConflictChoice: (path: string, modId: string) => void;
  onEnableMod: (modId: string) => void;
  onDeployPathOverride: (path: string | undefined) => void;
  onLoadoutChange: (loadout: Loadout) => void;
  onModsUpdated: (mods: ModManifest[]) => void;
  onModsReordered?: (mods: ModManifest[]) => void;
  onDeployComplete: () => void;
  onDownloadComplete?: (gameId: string, mods: ModManifest[], stagingDir: string) => void;
  onNavigate: (view: ShellView) => void;
  onGamesLoaded?: (games: DetectedGame[]) => void;
  nxmNotice?: string | null;
  onClearNxmNotice?: () => void;
};

export function MainContent({
  view,
  game,
  games,
  gamesLoading = false,
  queue,
  deployRefreshKey,
  onSelectGame,
  onToggle,
  onRemove,
  onIngested,
  onConflictChoice,
  onEnableMod,
  onDeployPathOverride,
  onLoadoutChange,
  onModsUpdated,
  onModsReordered,
  onDeployComplete,
  onDownloadComplete,
  onNavigate,
  onGamesLoaded,
  nxmNotice,
  onClearNxmNotice,
}: Props) {
  const [ignoreRequirements, setIgnoreRequirements] = useState(false);
  const [issue, setIssue] = useState<UserFacingIssue | null>(null);
  const driftPrompted = useRef<string | null>(null);

  useEffect(() => {
    driftPrompted.current = null;
  }, [deployRefreshKey]);

  useEffect(() => {
    if (!game || driftPrompted.current === game.id) return;
    getDeployState(game.id)
      .then((response) => {
        if (!response) return;
        const { state } = response;
        const outOfSync =
          !state.report.verified ||
          state.report.missing > 0 ||
          state.report.mismatched > 0 ||
          response.driftDetected;
        if (!outOfSync || state.manifest.targets.length === 0) return;
        driftPrompted.current = game.id;
        setIssue({
          id: "drift-detected",
          title: "Installed mods changed on disk",
          explanation:
            "Files in your game folder no longer match the last Supervisor deploy. Something may have edited or removed them outside the app.",
          impact: "Open Mods to purge and redeploy, or verify game files in Steam.",
          choices: [
            {
              id: "open-mods",
              label: "Open Mods",
              description: "Review, purge, or redeploy",
              recommended: true,
            },
            { id: "acknowledge", label: "Dismiss", description: "Ignore for now", recommended: false },
          ],
        });
      })
      .catch(console.error);
  }, [game, deployRefreshKey]);

  const handlePaths = async (paths: string[]) => {
    if (!game || paths.length === 0) return;
    try {
      const result = await ingestForGame(game.id, paths);
      onIngested(result.mods, result.stagingDir);
      onNavigate("mods");
    } catch (e) {
      console.error(e);
    }
  };

  useWindowDrop(handlePaths, (view === "mods" || view === "collections") && !!game);

  const handleIssueChoice = (choiceId: string) => {
    if (choiceId === "open-mods") {
      onNavigate("mods");
      setIssue(null);
      return;
    }
    setIssue(null);
  };

  const needsGame = view === "mods" || view === "collections" || view === "plugins";

  if (needsGame && !game) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-3 p-8 text-center">
        <p className="text-sm text-text-secondary">Select a game to continue.</p>
        <button
          type="button"
          onClick={() => onNavigate("games")}
          className="text-sm text-primary hover:underline"
        >
          Go to Games
        </button>
      </div>
    );
  }

  switch (view) {
    case "home":
      return <WelcomeScreen />;

    case "downloads":
      return (
        <>
          <DownloadsPanel
            gameId={game?.id}
            nexusDomain={game?.nexusDomain}
            onDownloadComplete={onDownloadComplete}
          />
          <IssueModal issue={issue} onClose={() => setIssue(null)} onChoice={handleIssueChoice} />
        </>
      );

    case "games":
      return (
        <LibraryPage
          embedded
          games={games}
          gamesLoading={gamesLoading}
          selectedId={game?.id}
          onSelectGame={(g) => void onSelectGame(g)}
          onGamesLoaded={onGamesLoaded}
        />
      );

    case "settings":
      return (
        <SettingsPage
          embedded
          selectedGame={game}
          games={games}
          gamesLoading={gamesLoading}
          onSelectGame={onSelectGame}
          onGamesLoaded={onGamesLoaded}
          nxmNotice={nxmNotice}
          onClearNxmNotice={onClearNxmNotice}
        />
      );

    case "collections":
      return (
        <CollectionsPage
          embedded
          games={games}
          selectedGame={game}
          onIngested={onIngested}
        />
      );

    case "plugins":
    case "mods":
      return game ? (
        <>
          <ModsTab
            game={game}
            queue={queue}
            defaultShowPlugins={view === "plugins"}
            onToggle={onToggle}
            onRemove={onRemove}
            onIngested={onIngested}
            onConflictChoice={(issueId, choiceId) => {
              if (choiceId === "cancel") return;
              onConflictChoice(issueId.replace(/^conflict-/, ""), choiceId.replace(/^prefer-/, ""));
            }}
            onEnableMod={onEnableMod}
            onDeployComplete={onDeployComplete}
            onLoadoutChange={onLoadoutChange}
            onModsUpdated={onModsUpdated}
            onModsReordered={onModsReordered}
            onDeployPathOverride={onDeployPathOverride}
            deployRefreshKey={deployRefreshKey}
            ignoreRequirements={ignoreRequirements}
            onIgnoreRequirements={() => setIgnoreRequirements(true)}
            onNavigate={onNavigate}
          />
          <IssueModal issue={issue} onClose={() => setIssue(null)} onChoice={handleIssueChoice} />
        </>
      ) : null;

    default:
      return null;
  }
}
