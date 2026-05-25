/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useEffect } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ingestModPaths } from "@/lib/api/mods";
import { isTauri } from "@/lib/env";

export function useWindowDrop(onDrop: (paths: string[]) => void, enabled = true) {
  useEffect(() => {
    if (!isTauri() || !enabled) return;

    let unlisten: (() => void) | undefined;
    getCurrentWindow()
      .onDragDropEvent((event) => {
        if (event.payload.type === "drop" && event.payload.paths.length > 0) {
          onDrop(event.payload.paths);
        }
      })
      .then((fn) => {
        unlisten = fn;
      });

    return () => unlisten?.();
  }, [enabled, onDrop]);
}

export async function pickModPaths(): Promise<string[]> {
  if (!isTauri()) return [];

  const selected = await open({
    multiple: true,
    directory: false,
    filters: [{ name: "Mod files", extensions: ["zip", "7z", "rar"] }],
  });

  if (!selected) return [];
  return Array.isArray(selected) ? selected : [selected];
}

export async function ingestForGame(gameId: string, paths: string[]) {
  return ingestModPaths(gameId, paths);
}
