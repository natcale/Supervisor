/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { Plus, Gamepad2 } from "lucide-react";
import type { DetectedGame, ShellView } from "@/types";
import { steamCoverUrl, withCover } from "@/types";

type Props = {
  games: DetectedGame[];
  selectedGame?: DetectedGame | null;
  onSelectGame: (game: DetectedGame) => void;
  onNavigate: (view: ShellView) => void;
};

export function CompactGameSidebar({
  games,
  selectedGame,
  onSelectGame,
  onNavigate,
}: Props) {
  return (
    <aside
      data-theme-slot="shell.compactBar"
      className="flex h-full w-[72px] shrink-0 flex-col items-center bg-panel"
    >
      <div className="relative min-h-0 w-full flex-1">
        <div className="compact-bar-scroll flex h-full flex-col items-center gap-3 overflow-y-auto px-2 pb-6 pt-1">
          {games.map((game) => {
            const enriched = withCover(game);
            const icon = enriched.coverUrl ?? steamCoverUrl(game.appId);
            const selected = selectedGame?.id === game.id;
            return (
              <button
                key={game.id}
                type="button"
                title={game.name}
                onClick={() => onSelectGame(game)}
                className={`group relative h-12 w-12 shrink-0 overflow-hidden rounded-lg transition-colors ${
                  selected
                    ? "rounded-lg ring-2 ring-text-focus ring-offset-2 ring-offset-panel"
                    : "hover:rounded-lg"
                }`}
              >
                {icon ? (
                  <img
                    src={icon}
                    alt=""
                    className="h-full w-full object-cover"
                  />
                ) : (
                  <span className="flex h-full w-full items-center justify-center bg-panel-hover text-text-muted">
                    <Gamepad2 size={18} />
                  </span>
                )}
              </button>
            );
          })}
        </div>
        <div
          className="compact-bar-fade pointer-events-none absolute inset-x-0 bottom-0 h-10"
          aria-hidden
        />
      </div>

      <div className="mt-2 flex shrink-0 flex-col items-center px-2 pb-1">
        <button
          type="button"
          title="Add game"
          onClick={() => onNavigate("games")}
          className="flex h-12 w-12 items-center justify-center rounded-lg border border-text-muted/50 text-text-secondary hover:bg-panel-hover hover:text-text-primary"
        >
          <Plus size={20} />
        </button>
      </div>
    </aside>
  );
}
