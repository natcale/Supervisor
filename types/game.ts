/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import { steamCoverUrl, steamHeroUrl } from "@/lib/steam";

export type GamePlatform = "steam" | "epic" | "gog" | "heroic" | "manual";

export interface DetectedGame {
  id: string;
  name: string;
  platform: GamePlatform;
  installPath: string;
  executable?: string;
  appId?: string;
  dataPath?: string;
  nexusDomain?: string;
  profileId?: string;
  coverUrl?: string;
}

export interface GameScanResult {
  games: DetectedGame[];
  scannedAt: number;
}

export type AppView = "game" | "library" | "settings" | "collections";

/** Carousel tile id for the add-game slot (always last). */
export const ADD_GAME_CAROUSEL_ID = "__add_game__";

/** Primary navigation — sidebar sections */
export type ShellView =
  | "home"
  | "downloads"
  | "games"
  | "settings"
  | "mods"
  | "collections"
  | "plugins";

export const GAME_VIEWS: ShellView[] = ["mods", "collections", "plugins"];

export function viewRequiresGame(view: ShellView): boolean {
  return GAME_VIEWS.includes(view);
}

/** @deprecated Use ShellView */
export type BottomTab = "home" | "mods" | "downloads";

export interface CarouselGame {
  id: string;
  title: string;
  image: string;
  background?: string;
  game?: DetectedGame;
  isAddGame?: boolean;
}

export const ADD_GAME_IMAGE = "/assets/media/add_game.png";

export function toAddGameCarouselItem(): CarouselGame {
  return {
    id: ADD_GAME_CAROUSEL_ID,
    title: "Add Game",
    image: ADD_GAME_IMAGE,
    isAddGame: true,
  };
}

export function toCarouselGame(game: DetectedGame): CarouselGame {
  const enriched = withCover(game);
  return {
    id: enriched.id,
    title: enriched.name,
    image: enriched.coverUrl ?? "",
    background: steamHeroUrl(enriched.appId),
    game: enriched,
  };
}

export function withCover(game: DetectedGame): DetectedGame {
  if (game.platform === "steam" && game.appId) {
    return { ...game, coverUrl: steamCoverUrl(game.appId) };
  }
  return game;
}

export function gameMatchesNxm(game: DetectedGame, domain: string): boolean {
  const needle = domain.toLowerCase();
  if (game.nexusDomain?.toLowerCase() === needle) return true;
  return game.name.toLowerCase().includes(needle);
}

export { steamCoverUrl, steamHeroUrl, steamLogoUrl, steamPageBackgroundUrl } from "@/lib/steam";
