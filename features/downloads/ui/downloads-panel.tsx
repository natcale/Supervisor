/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { EmptyState } from "@/features/shell/ui/empty-state";
import { RefreshCw, X } from "lucide-react";
import { isTauri } from "@/lib/env";
import { Button } from "@/components/ui/button";
import { useDownloadQueue } from "@/features/downloads/model/use-download-queue";
import {
  downloadJobImage,
  isActiveDownload,
  statusLabel,
} from "@/features/downloads/lib/download-utils";

type Props = {
  gameId?: string;
  nexusDomain?: string;
};

export function DownloadsPanel({ gameId, nexusDomain }: Props) {
  const {
    jobs,
    active,
    loading,
    clearing,
    startingPending,
    hasQueuedManual,
    refresh,
    startQueued,
    clearFinished,
    cancelJob,
  } = useDownloadQueue(gameId);

  if (!isTauri()) {
    return (
      <div className="flex h-full items-center justify-center p-6 text-sm text-text-secondary">
        Downloads are available in the desktop app.
      </div>
    );
  }

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex shrink-0 items-center justify-between px-3 py-3">
        <div className="flex items-center gap-2 text-lg font-light text-text-primary">
          Downloads
          {active.length > 0 && (
            <span className="rounded-full bg-accent px-2 py-0.5 text-xs text-white">
              {active.length} active
            </span>
          )}
        </div>
        <div className="flex items-center gap-2">
          {jobs.length > active.length && (
            <Button
              variant="ghost"
              size="sm"
              className="h-7 text-xs text-error hover:text-error"
              disabled={clearing}
              onClick={() => void clearFinished()}
            >
              {clearing ? "Clearing…" : "Clear finished"}
            </Button>
          )}
          {hasQueuedManual && (
            <Button
              variant="secondary"
              size="sm"
              className="h-7 text-xs"
              disabled={startingPending}
              onClick={() => void startQueued()}
            >
              {startingPending ? "Starting…" : "Start pending"}
            </Button>
          )}
          <Button variant="ghost" size="sm" className="h-7" onClick={refresh}>
            <RefreshCw size={14} className={loading ? "animate-spin" : ""} />
          </Button>
        </div>
      </div>

      {jobs.length === 0 ? (
        <EmptyState
          slotId="view.downloads.empty"
          iconSrc="/assets/icons/downloads.svg"
          iconWidth={98}
          iconHeight={111}
          title="No Downloads Yet"
          message=""
          link={{
            label: "Go to Nexus",
            href: nexusDomain
              ? `https://www.nexusmods.com/${nexusDomain}`
              : "https://www.nexusmods.com/",
          }}
          align="center"
        />
      ) : (
        <ul className="min-h-0 flex-1 overflow-y-auto">
          {jobs.map((job) => {
            const image = downloadJobImage(job);
            const activeJob = isActiveDownload(job);
            return (
              <li
                key={job.id}
                className="flex items-center gap-3 border-b border-border px-4 py-3"
              >
                <div className="h-12 w-12 shrink-0 overflow-hidden rounded-md bg-panel-hover">
                  {image ? (
                    <img src={image} alt="" className="h-full w-full object-cover" />
                  ) : (
                    <div className="flex h-full w-full items-center justify-center text-xs text-text-muted">
                      mod
                    </div>
                  )}
                </div>
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm text-text-primary">{job.modName}</p>
                  <p className="text-xs text-text-muted">
                    {statusLabel(job.status)}
                    {activeJob && job.progress > 0 && ` · ${job.progress}%`}
                  </p>
                  {job.error && (
                    <p className="mt-0.5 text-xs text-[var(--error)]">{job.error}</p>
                  )}
                  {activeJob && job.progress > 0 && (
                    <div className="mt-1.5 h-1 overflow-hidden rounded-full bg-panel-hover">
                      <div
                        className="h-full bg-primary transition-all"
                        style={{ width: `${job.progress}%` }}
                      />
                    </div>
                  )}
                </div>
                {activeJob && (
                  <button
                    type="button"
                    className="shrink-0 text-text-muted hover:text-[var(--error)]"
                    onClick={() => void cancelJob(job.id)}
                    aria-label="Cancel download"
                  >
                    <X size={16} />
                  </button>
                )}
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}
