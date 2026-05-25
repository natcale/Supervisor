/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useEffect, useState } from "react";
import type { Update } from "@tauri-apps/plugin-updater";
import { Button } from "@/components/ui/button";
import { checkForUpdates, installUpdate } from "@/lib/api/updates";
import { getAppSettings } from "@/lib/api/settings";
import { mergeSettings } from "@/lib/settings-defaults";
import { isTauri } from "@/lib/env";
import { formatInvokeError } from "@/lib/errors";

export function AppUpdateBanner() {
  const [update, setUpdate] = useState<Update | null>(null);
  const [installing, setInstalling] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!isTauri()) return;
    void (async () => {
      const settings = mergeSettings(await getAppSettings());
      if (settings.updateCheckMode !== "onStartup") return;
      try {
        const available = await checkForUpdates();
        if (available) setUpdate(available);
      } catch {
        // Background startup check — ignore network/updater errors.
      }
    })();
  }, []);

  if (!update) return null;

  return (
    <div className="flex shrink-0 items-center justify-between gap-3 border-b border-primary/30 bg-primary/10 px-4 py-2 text-sm text-text-primary">
      <span>
        Supervisor {update.version} is available.
        {typeof update.body === "string" && update.body ? ` ${update.body}` : ""}
      </span>
      <div className="flex items-center gap-2">
        {error && <span className="text-xs text-error">{error}</span>}
        <Button
          variant="accent"
          size="sm"
          disabled={installing}
          onClick={() => {
            setInstalling(true);
            setError(null);
            void installUpdate(update).catch((e) => {
              setError(formatInvokeError(e));
              setInstalling(false);
            });
          }}
        >
          {installing ? "Installing…" : "Update and restart"}
        </Button>
      </div>
    </div>
  );
}
