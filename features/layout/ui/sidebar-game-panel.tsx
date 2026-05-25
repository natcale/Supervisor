/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useState } from "react";
import { launchGame } from "@/lib/api/games";
import { IssueModal } from "@/components/ui/issue-modal";
import type { DetectedGame, UserFacingIssue } from "@/types";
import { steamCoverUrl, withCover } from "@/types";
type Props = {
  game: DetectedGame | null;
  onPickGame?: () => void;
};

export function SidebarGamePanel({ game, onPickGame }: Props) {
  const [playing, setPlaying] = useState(false);
  const [issue, setIssue] = useState<UserFacingIssue | null>(null);

  if (!game) {
    return (
      <div className="shrink-0 p-3">
        <p className="mb-2 text-xs text-text-muted">No game selected</p>
        {onPickGame && (
          <button
            type="button"
            onClick={onPickGame}
            className="w-full rounded bg-panel-hover px-3 py-2 text-xs text-text-secondary hover:text-text-primary"
          >
            Choose a game
          </button>
        )}
      </div>
    );
  }

  const enriched = withCover(game);
  const cover = enriched.coverUrl ?? steamCoverUrl(game.appId);

  const play = async () => {
    setPlaying(true);
    try {
      await launchGame(game);
    } catch (e) {
      if (e && typeof e === "object" && "title" in e) setIssue(e as UserFacingIssue);
    } finally {
      setPlaying(false);
    }
  };

  return (
    <>
      <div className="shrink-0 p-2">
        <p className="mb-2 px-1 truncate text-sm font-medium text-text-primary">{game.name}</p>
        {cover ? (
          <img src={cover} alt="" className="aspect-460/215 w-full rounded-lg object-cover" />
        ) : (
          <div className="flex aspect-460/215 w-full items-center justify-center rounded-lg bg-panel-secondary text-center text-xs text-text-muted">
            {game.name}
          </div>
        )}
        <button
          type="button"
          onClick={() => void play()}
          disabled={playing}
          className="mt-3 flex h-8 w-full items-center justify-center gap-1.5 rounded-lg bg-primary text-sm text-white hover:bg-primary-hover disabled:opacity-60"
        >
          Play
        </button>
      </div>

      <IssueModal issue={issue} onClose={() => setIssue(null)} onChoice={() => setIssue(null)} />
    </>
  );
}
