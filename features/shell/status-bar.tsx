/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { showAboutDialog } from "@/lib/api/app";
import { getDeployState } from "@/lib/tauri";
import { isTauri } from "@/lib/env";
import { useDownloadQueue } from "@/features/downloads/model/use-download-queue";
import type { DetectedGame, GameQueue, ShellView } from "@/types";

type Props = {
  shellView: ShellView;
  game: DetectedGame | null;
  queue: GameQueue;
  deployRefreshKey?: number;
  onNavigate: (view: ShellView) => void;
  nxmStatus?: string | null;
  onClearNxmStatus?: () => void;
};

export function StatusBar({
  shellView,
  game,
  queue,
  deployRefreshKey = 0,
  onNavigate,
  nxmStatus,
  onClearNxmStatus,
}: Props) {
  const [deploySummary, setDeploySummary] = useState<string | null>(null);
  const [drift, setDrift] = useState(false);
  const [appVersion, setAppVersion] = useState<string | null>(null);
  const { active: activeDownloads } = useDownloadQueue(game?.id);

  useEffect(() => {
    if (!isTauri()) return;
    void getVersion()
      .then((version) => setAppVersion(`v${version}`))
      .catch(() => setAppVersion(null));
  }, []);

  useEffect(() => {
    if (!isTauri() || !game) {
      setDeploySummary(null);
      setDrift(false);
      return;
    }
    let cancelled = false;
    getDeployState(game.id)
      .then((response) => {
        if (cancelled) return;
        if (!response) {
          setDeploySummary(null);
          setDrift(false);
          return;
        }
        const { state } = response;
        const outOfSync =
          !state.report.verified ||
          state.report.missing > 0 ||
          state.report.mismatched > 0;
        setDrift(outOfSync || response.driftDetected);
        setDeploySummary(
          state.report.verified
            ? `${state.report.linked} files verified`
            : `${state.report.linked} linked · ${state.report.missing} missing${state.report.mismatched > 0 ? ` · ${state.report.mismatched} mismatched` : ""}`,
        );
      })
      .catch(() => {
        if (!cancelled) {
          setDeploySummary(null);
          setDrift(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [game, queue.enabledIds.length, queue.mods.length, deployRefreshKey]);

  const modSummary =
    queue.mods.length > 0
      ? `${queue.enabledIds.length}/${queue.mods.length} mods enabled`
      : null;

  const viewLabel = shellView.charAt(0).toUpperCase() + shellView.slice(1);

  return (
    <footer className="flex h-6 shrink-0 items-center gap-0 overflow-hidden bg-statusbar text-xs text-statusbar-fg" data-theme-slot="shell.statusbar">
      <StatusSegment title={game?.name ?? "No game"} onClick={() => onNavigate("games")}>
        {game?.name ?? "No game selected"}
      </StatusSegment>

      <StatusSegment title={viewLabel}>{viewLabel}</StatusSegment>

      {queue.loadoutName && (
        <StatusSegment title={`Loadout: ${queue.loadoutName}`}>{queue.loadoutName}</StatusSegment>
      )}

      {modSummary && (
        <StatusSegment title={modSummary} onClick={() => onNavigate("mods")}>
          {modSummary}
        </StatusSegment>
      )}

      {deploySummary && (
        <StatusSegment
          title={drift ? "Deployed files changed on disk" : deploySummary}
          onClick={() => onNavigate("mods")}
          className={drift ? "bg-[var(--warning)]/30" : undefined}
        >
          {drift ? "Drift detected" : deploySummary}
        </StatusSegment>
      )}

      {activeDownloads.length > 0 && (
        <StatusSegment title="Active downloads" onClick={() => onNavigate("downloads")}>
          {activeDownloads.length} downloading
        </StatusSegment>
      )}

      {nxmStatus && (
        <StatusSegment
          title={nxmStatus}
          onClick={() => {
            onNavigate("downloads");
            onClearNxmStatus?.();
          }}
          className="max-w-none flex-1 bg-[var(--info)]/20"
        >
          {nxmStatus}
        </StatusSegment>
      )}

      <StatusSegment title="Deploy method: hardlink">Hardlink deploy</StatusSegment>

      {appVersion && (
        <StatusSegment
          className="ml-auto shrink-0"
          title="About Supervisor"
          onClick={() => void showAboutDialog()}
        >
          {appVersion}
        </StatusSegment>
      )}
    </footer>
  );
}

function StatusSegment({
  children,
  title,
  onClick,
  className,
}: {
  children: React.ReactNode;
  title: string;
  onClick?: () => void;
  className?: string;
}) {
  const base =
    "flex h-full max-w-[220px] shrink-0 items-center truncate px-2";
  if (onClick) {
    return (
      <button
        type="button"
        title={title}
        onClick={onClick}
        className={`${base} hover:bg-white/10 ${className ?? ""}`}
      >
        {children}
      </button>
    );
  }
  return (
    <span title={title} className={`${base} ${className ?? ""}`}>
      {children}
    </span>
  );
}
