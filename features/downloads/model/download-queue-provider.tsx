/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { listen } from "@tauri-apps/api/event";
import {
  cancelDownload,
  clearFinishedDownloads,
  getDownloadQueue,
  startPendingDownloads,
} from "@/lib/api/downloads";
import { getAppSettings } from "@/lib/api/settings";
import { mergeSettings } from "@/lib/settings-defaults";
import { isTauri } from "@/lib/env";
import type { DownloadJob, ModManifest } from "@/types";
import {
  dedupeDownloadJobs,
  isActiveDownload,
} from "@/features/downloads/lib/download-utils";

type DownloadQueueContextValue = {
  jobs: DownloadJob[];
  active: DownloadJob[];
  activeCount: number;
  overallProgress: number;
  loading: boolean;
  clearing: boolean;
  startingPending: boolean;
  hasQueuedManual: boolean;
  refresh: () => void;
  startQueued: () => Promise<void>;
  clearFinished: () => Promise<void>;
  cancelJob: (jobId: string) => Promise<void>;
  jobsForGame: (gameId?: string) => DownloadJob[];
};

const DownloadQueueContext = createContext<DownloadQueueContextValue | null>(null);

export function DownloadQueueProvider({
  children,
  onDownloadComplete,
}: {
  children: ReactNode;
  onDownloadComplete?: (gameId: string, mods: ModManifest[], stagingDir: string) => void;
}) {
  const [jobs, setJobs] = useState<DownloadJob[]>([]);
  const [loading, setLoading] = useState(true);
  const [autoStartDownloads, setAutoStartDownloads] = useState(true);
  const [startingPending, setStartingPending] = useState(false);
  const [clearing, setClearing] = useState(false);

  const syncFromBackend = useCallback(() => {
    if (!isTauri()) {
      setJobs([]);
      setLoading(false);
      return;
    }
    void getDownloadQueue().then((list) => {
      setJobs(dedupeDownloadJobs(list));
      setLoading(false);
    });
  }, []);

  useEffect(() => {
    void getAppSettings()
      .then((s) => setAutoStartDownloads(mergeSettings(s).autoStartDownloads))
      .catch(console.error);
  }, []);

  useEffect(() => {
    if (!isTauri()) {
      setLoading(false);
      return;
    }

    syncFromBackend();

    const unsubs: Array<() => void> = [];
    listen("download://updated", syncFromBackend).then((u) => unsubs.push(u));
    listen("download://queue-changed", syncFromBackend).then((u) => unsubs.push(u));
    listen<{ jobId: string; gameId: string; mods: Record<string, unknown>[]; stagingDir: string }>(
      "download://completed",
      (e) => {
        onDownloadComplete?.(
          e.payload.gameId,
          e.payload.mods.map((m) => ({
            id: String(m.id),
            name: String(m.name),
            slug: m.slug as string | undefined,
            files: (m.files as string[]) ?? [],
            dependencies: (m.dependencies as string[]) ?? [],
            installState: m.installState as ModManifest["installState"],
            needsFomod: Boolean(m.needsFomod),
            nexus: m.nexus as ModManifest["nexus"],
          })),
          e.payload.stagingDir,
        );
        syncFromBackend();
      },
    ).then((u) => unsubs.push(u));

    return () => unsubs.forEach((u) => u());
  }, [onDownloadComplete, syncFromBackend]);

  const active = useMemo(() => jobs.filter(isActiveDownload), [jobs]);
  const activeCount = active.length;
  const overallProgress = useMemo(() => {
    if (active.length === 0) return 0;
    const sum = active.reduce((acc, j) => acc + j.progress, 0);
    return Math.round(sum / active.length);
  }, [active]);

  const hasQueuedManual = !autoStartDownloads && jobs.some((j) => j.status === "queued");

  const jobsForGame = useCallback(
    (gameId?: string) => (gameId ? jobs.filter((j) => j.gameId === gameId) : jobs),
    [jobs],
  );

  const startQueued = useCallback(async () => {
    setStartingPending(true);
    try {
      await startPendingDownloads();
      syncFromBackend();
    } catch (e) {
      console.error(e);
    } finally {
      setStartingPending(false);
    }
  }, [syncFromBackend]);

  const clearFinished = useCallback(async () => {
    setClearing(true);
    try {
      await clearFinishedDownloads();
      syncFromBackend();
    } catch (e) {
      console.error(e);
    } finally {
      setClearing(false);
    }
  }, [syncFromBackend]);

  const cancelJob = useCallback(
    async (jobId: string) => {
      await cancelDownload(jobId);
      syncFromBackend();
    },
    [syncFromBackend],
  );

  const value = useMemo(
    () => ({
      jobs,
      active,
      activeCount,
      overallProgress,
      loading,
      clearing,
      startingPending,
      hasQueuedManual,
      refresh: syncFromBackend,
      startQueued,
      clearFinished,
      cancelJob,
      jobsForGame,
    }),
    [
      jobs,
      active,
      activeCount,
      overallProgress,
      loading,
      clearing,
      startingPending,
      hasQueuedManual,
      syncFromBackend,
      startQueued,
      clearFinished,
      cancelJob,
      jobsForGame,
    ],
  );

  return <DownloadQueueContext.Provider value={value}>{children}</DownloadQueueContext.Provider>;
}

export function useDownloadQueue(gameId?: string) {
  const ctx = useContext(DownloadQueueContext);
  if (!ctx) {
    return {
      jobs: [] as DownloadJob[],
      active: [] as DownloadJob[],
      activeCount: 0,
      overallProgress: 0,
      loading: false,
      clearing: false,
      startingPending: false,
      hasQueuedManual: false,
      refresh: () => {},
      startQueued: async () => {},
      clearFinished: async () => {},
      cancelJob: async () => {},
    };
  }

  if (!gameId) return ctx;

  const scoped = ctx.jobsForGame(gameId);
  const active = scoped.filter(isActiveDownload);
  return {
    ...ctx,
    jobs: scoped,
    active,
    activeCount: active.length,
    overallProgress:
      active.length === 0
        ? 0
        : Math.round(active.reduce((acc, j) => acc + j.progress, 0) / active.length),
    hasQueuedManual: !ctx.hasQueuedManual ? false : scoped.some((j) => j.status === "queued"),
  };
}

export function useDownloadQueueContext() {
  const ctx = useContext(DownloadQueueContext);
  if (!ctx) {
    throw new Error("useDownloadQueueContext requires DownloadQueueProvider");
  }
  return ctx;
}
