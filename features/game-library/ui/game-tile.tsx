/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useState } from "react";
import { cn } from "@/lib/cn";
import { TILE_WIDTH } from "@/features/game-library/lib/carousel-config";
import type { CarouselGame } from "@/types";

type TilePosition = "left" | "right" | "selected" | "none";

type Props = {
  item: CarouselGame;
  position: TilePosition;
  onSelect?: () => void;
};

export function GameTile({ item, position, onSelect }: Props) {
  const [imageFailed, setImageFailed] = useState(false);
  const selected = position === "selected";

  return (
    <div className="relative shrink-0 overflow-visible" style={{ width: TILE_WIDTH }}>
      <div className="relative aspect-[92/43] w-full overflow-visible">
        <button
          type="button"
          className={cn(
            "absolute inset-0 origin-top outline-none",
            selected ? "z-10 scale-[1.15] opacity-100" : "scale-100 opacity-90",
          )}
          onClick={onSelect}
          aria-label={item.title}
        >
          <div
            className={cn(
              "ps-focus-ring inset-[-5px] transition-opacity duration-300",
              selected ? "opacity-100" : "opacity-0",
            )}
          />

          <div className="relative h-full w-full overflow-hidden bg-card">
            {item.image && !imageFailed ? (
              <img
                src={item.image}
                alt={item.title}
                className="h-full w-full object-cover"
                onError={() => setImageFailed(true)}
              />
            ) : (
              <div className="flex h-full w-full items-center justify-center bg-card p-2 text-center text-xs font-medium uppercase text-white/50">
                {item.title}
              </div>
            )}

            {selected && <div key={item.id} className="tile-sheen pointer-events-none" />}

            <div
              className={cn(
                "pointer-events-none absolute inset-0 bg-black/30",
                position === "none" ? "opacity-100" : "opacity-0",
              )}
            />
            <div
              className={cn(
                "pointer-events-none absolute inset-0 bg-linear-to-r from-black/90 to-transparent",
                position === "right" ? "opacity-100" : "opacity-0",
              )}
            />
            <div
              className={cn(
                "pointer-events-none absolute inset-0 bg-linear-to-l from-black/90 to-transparent",
                position === "left" ? "opacity-100" : "opacity-0",
              )}
            />
          </div>
        </button>

        {selected && (
          <h2
            className="tile-title-in pointer-events-none absolute z-20 w-max whitespace-nowrap text-lg font-light text-foreground"
            style={{ left: "calc(104% + 1.6rem)", bottom: 0, transform: "translateY(115%)" }}
          >
            {item.title}
          </h2>
        )}
      </div>
    </div>
  );
}
