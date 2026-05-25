/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useEffect, useState } from "react";
import { getDeployState } from "@/lib/api/deploy";
import type { PersistedDeployState } from "@/types";
import { ChevronDown, ChevronUp, FolderOpen } from "lucide-react";

type Props = {
  gameId: string;
  liveResult?: PersistedDeployState | null;
};

export function DeployStatus({ gameId, liveResult }: Props) {
  const [state, setState] = useState<PersistedDeployState | null>(liveResult ?? null);
  const [expanded, setExpanded] = useState(false);

  useEffect(() => {
    if (liveResult) {
      setState(liveResult);
      return;
    }
    let cancelled = false;
    getDeployState(gameId).then((result) => {
      if (!cancelled) setState(result?.state ?? null);
    });
    return () => {
      cancelled = true;
    };
  }, [gameId, liveResult]);

  if (!state) return null;

  const { report, manifest, profileName, primaryModPath } = state;
  const when = new Date(manifest.deployedAt * 1000).toLocaleString();

  return (
    <div className="rounded-md border border-border bg-panel-secondary p-4 text-sm">
      <div className="flex items-start justify-between gap-3">
        <div>
          <p
            className={
              report.verified ? "text-[var(--success)]" : "text-[var(--warning)]"
            }
          >
            {report.verified
              ? `Verified: ${report.linked} file(s) linked into ${primaryModPath}`
              : `Partial: ${report.linked} linked, ${report.missing} missing, ${report.mismatched} mismatched`}
          </p>
          <p className="mt-1 text-xs text-text-muted">
            Profile: {profileName} · Last deploy: {when}
          </p>
          {report.profileWarning && (
            <p className="mt-1 text-xs text-[var(--warning)]">{report.profileWarning}</p>
          )}
        </div>
        <button
          type="button"
          className="shrink-0 text-text-muted hover:text-text-primary"
          onClick={() => setExpanded((v) => !v)}
          aria-expanded={expanded}
        >
          {expanded ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
        </button>
      </div>

      {expanded && (
        <div className="mt-3 max-h-40 space-y-1 overflow-y-auto border-t border-border pt-3">
          {manifest.targets.slice(0, 50).map((t) => (
            <div
              key={`${t.deployRoot}/${t.relPath}`}
              className="flex items-center gap-2 text-xs text-text-secondary"
            >
              <FolderOpen size={12} className="shrink-0 opacity-60" />
              <span className="truncate">{t.relPath}</span>
              <span className="shrink-0 text-text-muted">→ {t.modType}</span>
            </div>
          ))}
          {manifest.targets.length > 50 && (
            <p className="text-xs text-text-muted">
              +{manifest.targets.length - 50} more files
            </p>
          )}
        </div>
      )}
    </div>
  );
}
