/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import { listen } from "@tauri-apps/api/event";
import type { AppSettings } from "@/types";
import { isTauri } from "@/lib/env";

export const SETTINGS_CHANGED_EVENT = "settings://changed";

export function listenSettingsChanged(
  handler: (settings: AppSettings) => void,
): Promise<() => void> {
  if (!isTauri()) {
    return Promise.resolve(() => {});
  }
  return listen<AppSettings>(SETTINGS_CHANGED_EVENT, (event) => {
    handler(event.payload);
  });
}
