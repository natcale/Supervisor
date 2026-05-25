/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { NxmPayload } from "@/types";
import { Download } from "lucide-react";

interface BannerProps {
  onModLink?: (payload: Extract<NxmPayload, { kind: "modDownload" }>) => void;
}

export function Banner({ onModLink }: BannerProps) {
  const [payload, setPayload] = useState<NxmPayload | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen<NxmPayload>("nxm://received", (event) => {
      setPayload(event.payload);
      if (event.payload.kind === "modDownload") {
        onModLink?.(event.payload);
      }
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, [onModLink]);

  if (!payload) return null;

  if (payload.kind === "modDownload") {
    return (
      <div className="flex items-center gap-3 border-b border-border bg-panel-secondary px-4 py-3">
        <Download size={18} className="text-[var(--info)]" />
        <div>
          <p className="text-sm text-text-primary">
            Nexus Mods sent a download for{" "}
            <span className="font-medium capitalize">{payload.gameDomain}</span>
          </p>
          <p className="text-xs text-text-muted">
            Mod #{payload.modId} · File #{payload.fileId}
          </p>
        </div>
      </div>
    );
  }

  return null;
}
