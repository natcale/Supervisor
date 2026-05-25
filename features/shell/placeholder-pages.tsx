/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useState } from "react";
import { Carousel } from "@/features/game-library/ui/carousel";
import { addManualGame } from "@/lib/api/games";
import { importVortexCollection, installCollectionMods, type CollectionImportResult } from "@/lib/api/collections";
import { ingestForGame, pickModPaths } from "@/lib/drop";
import { Button } from "@/components/ui/button";
import type { DetectedGame, ModManifest } from "@/types";
import { steamCoverUrl, withCover } from "@/types";
import { FolderOpen, Package } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";

type LibraryPageProps = {
  games: DetectedGame[];
  gamesLoading?: boolean;
  selectedId?: string;
  embedded?: boolean;
  onBack?: () => void;
  onSelectGame: (game: DetectedGame) => void;
  onGamesLoaded?: (games: DetectedGame[]) => void;
};

export function LibraryPage({
  games,
  gamesLoading = false,
  selectedId,
  onSelectGame,
  onGamesLoaded,
}: LibraryPageProps) {
  const addManual = async () => {
    const path = await open({ directory: true, multiple: false });
    if (!path || typeof path !== "string") return;
    try {
      const game = await addManualGame(path);
      onSelectGame(game);
      onGamesLoaded?.([...games.filter((g) => g.id !== game.id), game]);
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <div className="relative flex h-full min-h-0 flex-col overflow-hidden bg-content-panel">
      <Carousel
        games={games}
        gamesLoading={gamesLoading}
        selectedId={selectedId}
        onSelect={onSelectGame}
        onAddGame={() => void addManual()}
        onGamesLoaded={onGamesLoaded}
      />
    </div>
  );
}

type CollectionsPageProps = {
  games: DetectedGame[];
  selectedGame?: DetectedGame | null;
  embedded?: boolean;
  onBack?: () => void;
  onIngested: (mods: ModManifest[], stagingDir: string) => void;
};

export function CollectionsPage({
  games,
  selectedGame,
  embedded,
  onBack,
  onIngested,
}: CollectionsPageProps) {
  const [importing, setImporting] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [parsedCollection, setParsedCollection] = useState<CollectionImportResult | null>(null);

  const importArchives = async () => {
    if (!selectedGame) {
      setMessage("Select a game in Games first.");
      return;
    }
    const paths = await pickModPaths();
    if (paths.length === 0) return;
    setImporting(true);
    setMessage(null);
    try {
      const result = await ingestForGame(selectedGame.id, paths);
      onIngested(result.mods, result.stagingDir);
      setMessage(`Imported ${result.mods.length} mod archive(s) into ${selectedGame.name}.`);
    } catch (e) {
      setMessage(
        e && typeof e === "object" && "explanation" in e
          ? String((e as { explanation: string }).explanation)
          : "Import failed.",
      );
    } finally {
      setImporting(false);
    }
  };

  const importCollection = async () => {
    const path = await open({
      multiple: false,
      filters: [{ name: "Collection", extensions: ["collection", "zip"] }],
    });
    if (!path || typeof path !== "string") return;
    setImporting(true);
    setMessage(null);
    try {
      const parsed = await importVortexCollection(path);
      setParsedCollection(parsed);
      setMessage(
        `Parsed "${parsed.name}" with ${parsed.modCount} mod(s)` +
          (parsed.gameHint ? ` for ${parsed.gameHint}` : "") +
          ". Download mods from Nexus or add local archives.",
      );
    } catch (e) {
      setMessage(
        e && typeof e === "object" && "explanation" in e
          ? String((e as { explanation: string }).explanation)
          : "Collection import failed.",
      );
    } finally {
      setImporting(false);
    }
  };

  const installCollectionDownloads = async () => {
    if (!selectedGame) {
      setMessage("Select a game in Games first.");
      return;
    }
    if (!parsedCollection?.mods.length) {
      setMessage("Import a collection file first.");
      return;
    }
    setImporting(true);
    setMessage(null);
    try {
      const result = await installCollectionMods(selectedGame.id, parsedCollection.mods);
      setMessage(`Queued ${result.queued} download(s). Skipped ${result.skipped}.`);
    } catch (e) {
      setMessage(
        e && typeof e === "object" && "explanation" in e
          ? String((e as { explanation: string }).explanation)
          : "Install queue failed.",
      );
    } finally {
      setImporting(false);
    }
  };

  return (
    <div className="flex h-full min-h-0 flex-col overflow-hidden">
      <div className="flex shrink-0 items-center justify-between px-4 py-3">
        <h1 className="text-lg font-light text-text-primary">Collections</h1>
        {!embedded && onBack && (
          <Button variant="ghost" size="sm" onClick={onBack}>
            Back
          </Button>
        )}
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto p-4">
        <div className="mx-auto flex flex-col gap-6">
          <section className="rounded-xl bg-panel-secondary p-5" data-theme-slot="view.collections.vortex">
            <div className="flex items-start justify-between gap-6">
              <div className="min-w-0 flex-1">
                <div className="mb-3 flex items-center gap-2">
                  <Package size={18} className="text-primary" />
                  <h2 className="text-sm font-medium text-text-primary">Vortex collection</h2>
                </div>
                <p className="mb-4 text-sm text-text-secondary">
                  Import a Vortex .collection bundle to read its mod list and rules.
                </p>
                <Button variant="secondary" size="sm" disabled={importing} onClick={() => void importCollection()}>
                  Choose .collection file
                </Button>
                <Button
                  variant="accent"
                  size="sm"
                  className="ml-2"
                  disabled={importing || !parsedCollection?.mods.length || !selectedGame}
                  onClick={() => void installCollectionDownloads()}
                >
                  Queue Nexus downloads
                </Button>
              </div>
              <img
                src="/assets/vortex/logo.png"
                alt="Vortex"
                className="h-16 w-auto shrink-0 opacity-90"
              />
            </div>
          </section>

          <section className="rounded-xl bg-panel-secondary p-5">
            <div className="mb-3 flex items-center gap-2">
              <FolderOpen size={18} className="text-primary" />
              <h2 className="text-sm font-medium text-text-primary">Bulk import</h2>
            </div>
            <p className="mb-4 text-sm text-text-secondary">
              Import mod archives (.zip, .7z, .rar) for the selected game.
            </p>
            <p className="mb-4 text-xs text-text-muted">
              Target:{" "}
              <span className="text-text-primary">
                {selectedGame?.name ?? "No game selected"}
              </span>
            </p>
            <Button
              variant="accent"
              size="sm"
              disabled={importing || !selectedGame}
              onClick={() => void importArchives()}
            >
              <FolderOpen size={14} className="mr-1.5" />
              {importing ? "Importing…" : "Choose mod archives"}
            </Button>
            {message && <p className="mt-3 text-xs text-text-secondary">{message}</p>}
          </section>

          {games.length > 0 && (
            <section>
              <h2 className="mb-3 text-sm font-medium text-text-primary">Your games</h2>
              <div className="grid grid-cols-2 gap-3 sm:grid-cols-3">
                {games.map((game) => {
                  const enriched = withCover(game);
                  const cover = enriched.coverUrl ?? steamCoverUrl(game.appId);
                  const active = selectedGame?.id === game.id;
                  return (
                    <div
                      key={game.id}
                      className={`overflow-hidden ring-2 rounded-lg ring-transparent hover:ring-primary ${
                        active ? "ring-2 ring-primary" : "ring-transparent"
                      } bg-panel-secondary`}
                    >
                      {cover ? (
                        <img src={cover} alt="" className="aspect-[410/205] w-full object-cover" />
                      ) : (
                        <div className="flex aspect-[410/205] items-center justify-center text-sm text-text-muted">
                          {game.name.slice(0, 2)}
                        </div>
                      )}
                      <p className="truncate px-2 py-1.5 text-sm text-text-primary">
                        {game.name}
                      </p>
                    </div>
                  );
                })}
              </div>
            </section>
          )}
        </div>
      </div>
    </div>
  );
}
