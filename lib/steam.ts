/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
/** Steam CDN asset URLs (no API key required). */

export function steamHeroUrl(appId?: string): string | undefined {
  if (!appId) return undefined;
  return `https://cdn.cloudflare.steamstatic.com/steam/apps/${appId}/library_hero.jpg`;
}

export function steamLogoUrl(appId?: string): string | undefined {
  if (!appId) return undefined;
  return `https://cdn.cloudflare.steamstatic.com/steam/apps/${appId}/logo.png`;
}

export function steamCoverUrl(appId?: string): string | undefined {
  if (!appId) return undefined;
  return `https://cdn.cloudflare.steamstatic.com/steam/apps/${appId}/header.jpg`;
}

export function steamPageBackgroundUrl(appId?: string): string | undefined {
  if (!appId) return undefined;
  return `https://cdn.cloudflare.steamstatic.com/steam/apps/${appId}/page_bg_generated.jpg`;
}
