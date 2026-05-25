/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useEffect, useState } from "react";
import { getAppSettings } from "@/lib/api/settings";
import { listenSettingsChanged } from "@/lib/settings-events";
import { mergeSettings } from "@/lib/settings-defaults";
import { isTauri } from "@/lib/env";

export function DeveloperToolsShell() {
  const [enabled, setEnabled] = useState(false);

  useEffect(() => {
    if (!isTauri()) return;

    let active = true;
    void getAppSettings()
      .then((settings) => {
        if (active) setEnabled(mergeSettings(settings).developerTools);
      })
      .catch(() => {});

    let unlisten: (() => void) | undefined;
    void listenSettingsChanged((settings) => {
      setEnabled(mergeSettings(settings).developerTools);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      active = false;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    const body = document.body;
    body.dataset.developerTools = enabled ? "true" : "false";

    if (enabled) return;

    const blockContextMenu = (event: Event) => {
      event.preventDefault();
    };
    document.addEventListener("contextmenu", blockContextMenu);
    return () => {
      document.removeEventListener("contextmenu", blockContextMenu);
    };
  }, [enabled]);

  return null;
}
