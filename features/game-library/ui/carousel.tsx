/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { scanGames } from "@/lib/api/games";
import { GameTile } from "@/features/game-library/ui/game-tile";
import {
  CAROUSEL_EASE,
  CAROUSEL_MS,
  TILE_SLOT,
} from "@/features/game-library/lib/carousel-config";
import {
  toAddGameCarouselItem,
  toCarouselGame,
  type CarouselGame,
  type DetectedGame,
} from "@/types";
import { Button } from "@/components/ui/button";

function withAddTile(games: DetectedGame[]): CarouselGame[] {
  return [...games.map(toCarouselGame), toAddGameCarouselItem()];
}

type Props = {
  selectedId?: string;
  onSelect: (game: DetectedGame) => void;
  onAddGame?: () => void;
  onGamesLoaded?: (games: DetectedGame[]) => void;
  /** When provided, parent owns the game list (no internal scan). */
  games?: DetectedGame[];
  gamesLoading?: boolean;
};

export function Carousel({
  selectedId,
  onSelect,
  onAddGame,
  onGamesLoaded,
  games: gamesProp,
  gamesLoading = false,
}: Props) {
  const [items, setItems] = useState<CarouselGame[]>([toAddGameCarouselItem()]);
  const [loading, setLoading] = useState(gamesProp === undefined);
  const [error, setError] = useState<string | null>(null);
  const [focusedIndex, setFocusedIndex] = useState(0);
  const managedExternally = gamesProp !== undefined;

  useEffect(() => {
    if (!managedExternally) return;
    setItems(withAddTile(gamesProp));
    setLoading(gamesLoading);
    if (!gamesLoading) setError(null);
  }, [gamesProp, gamesLoading, managedExternally]);

  useEffect(() => {
    if (managedExternally) return;

    let cancelled = false;

    async function scanInitial() {
      setError(null);
      try {
        const result = await scanGames();
        if (cancelled) return;
        setItems(withAddTile(result.games));
        onGamesLoaded?.(result.games);
      } catch (e) {
        if (cancelled) return;
        setError(
          e && typeof e === "object" && "explanation" in e
            ? String((e as { explanation: string }).explanation)
            : "Could not scan for games.",
        );
      } finally {
        if (!cancelled) setLoading(false);
      }
    }

    void scanInitial();
    return () => {
      cancelled = true;
    };
  }, [managedExternally, onGamesLoaded]);

  useEffect(() => {
    if (items.length === 0) return;
    if (!selectedId) return;
    const index = items.findIndex((item) => item.game?.id === selectedId);
    if (index >= 0) setFocusedIndex(index);
  }, [items, selectedId]);

  const focused = items[focusedIndex];

  const navigate = useCallback(
    (index: number) => {
      const item = items[index];
      if (!item) return;
      setFocusedIndex(index);
      if (item.isAddGame) return;
      if (item.game) onSelect(item.game);
    },
    [items, onSelect],
  );

  const activateFocused = useCallback(() => {
    const item = items[focusedIndex];
    if (!item) return;
    if (item.isAddGame) {
      onAddGame?.();
      return;
    }
    if (item.game) onSelect(item.game);
  }, [focusedIndex, items, onAddGame, onSelect]);

  const handleTileClick = useCallback(
    (index: number) => {
      const item = items[index];
      if (!item) return;
      setFocusedIndex(index);
      if (item.isAddGame) return;
      if (item.game) onSelect(item.game);
    },
    [items, onSelect],
  );

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (items.length === 0) return;
      const target = event.target;
      if (
        target instanceof HTMLElement &&
        (target.isContentEditable ||
          target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.tagName === "SELECT")
      ) {
        return;
      }

      if (event.key === "ArrowRight") {
        event.preventDefault();
        navigate(Math.min(focusedIndex + 1, items.length - 1));
      } else if (event.key === "ArrowLeft") {
        event.preventDefault();
        navigate(Math.max(focusedIndex - 1, 0));
      } else if (event.key === "Enter") {
        event.preventDefault();
        activateFocused();
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [activateFocused, focusedIndex, items.length, navigate]);

  useEffect(() => {
    const onWheel = (event: WheelEvent) => {
      if (items.length === 0) return;
      const target = event.target;
      if (
        target instanceof HTMLElement &&
        (target.isContentEditable ||
          target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.tagName === "SELECT")
      ) {
        return;
      }

      if (Math.abs(event.deltaY) < Math.abs(event.deltaX)) return;
      if (event.deltaY === 0) return;

      event.preventDefault();
      if (event.deltaY > 0) {
        navigate(Math.min(focusedIndex + 1, items.length - 1));
      } else {
        navigate(Math.max(focusedIndex - 1, 0));
      }
    };

    window.addEventListener("wheel", onWheel, { passive: false });
    return () => window.removeEventListener("wheel", onWheel);
  }, [focusedIndex, items.length, navigate]);

  const background = useMemo(() => {
    if (focused?.isAddGame) return undefined;
    return focused?.background;
  }, [focused]);

  const retry = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await scanGames();
      setItems(withAddTile(result.games));
      onGamesLoaded?.(result.games);
    } catch (e) {
      setError(
        e && typeof e === "object" && "explanation" in e
          ? String((e as { explanation: string }).explanation)
          : "Could not scan for games.",
      );
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-text-secondary">
        Loading your library…
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-3 px-8 text-center">
        <p className="text-sm text-error">{error}</p>
        <Button variant="secondary" size="sm" onClick={retry}>
          Try again
        </Button>
      </div>
    );
  }

  return (
    <>
      {background && (
        <div className="absolute inset-0 z-0 overflow-hidden rounded-tl-xl">
          <img
            key={background}
            src={background}
            alt=""
            className="animate-fade h-full w-full object-cover"
          />
          <div className="absolute inset-0 bg-linear-to-r from-content-panel via-content-panel/70 to-transparent" />
        </div>
      )}

      <div className="relative z-10 flex h-full flex-col justify-start overflow-hidden pt-10">
        <div className="relative mb-6 mt-2 w-full overflow-visible pl-20">
          <div
            className="relative flex items-start gap-3 overflow-visible"
            style={{
              transform: `translateX(-${focusedIndex * TILE_SLOT}px)`,
              transition: `transform ${CAROUSEL_MS}ms ${CAROUSEL_EASE}`,
            }}
          >
            {items.map((item, index) => (
              <GameTile
                key={item.id}
                item={item}
                position={
                  index === focusedIndex
                    ? "selected"
                    : index === focusedIndex - 1
                      ? "left"
                      : index === focusedIndex + 1
                        ? "right"
                        : "none"
                }
                onSelect={() => handleTileClick(index)}
              />
            ))}
          </div>
        </div>
      </div>
    </>
  );
}
