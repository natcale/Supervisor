/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useEffect, useRef, useState, type CSSProperties, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { Loader2, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  downloadJobImage,
  isActiveDownload,
  statusLabel,
} from "@/features/downloads/lib/download-utils";
import { useDownloadQueue } from "@/features/downloads/model/use-download-queue";

type Placement = "right" | "bottom-end";

type Props = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  anchorRef: React.RefObject<HTMLElement | null>;
  placement?: Placement;
  gameId?: string;
  onViewAll?: () => void;
  children?: ReactNode;
};

const PANEL_WIDTH = 352;
const PANEL_MAX_HEIGHT = 420;
const VIEWPORT_PAD = 8;

export function DownloadPopup({
  open,
  onOpenChange,
  anchorRef,
  placement = "bottom-end",
  gameId,
  onViewAll,
  children,
}: Props) {
  const panelRef = useRef<HTMLDivElement>(null);
  const [panelStyle, setPanelStyle] = useState<CSSProperties>({ visibility: "hidden" });
  const [mounted, setMounted] = useState(false);
  const {
    jobs,
    active,
    activeCount,
    overallProgress,
    loading,
    clearing,
    hasQueuedManual,
    startingPending,
    refresh,
    startQueued,
    clearFinished,
    cancelJob,
  } = useDownloadQueue(gameId);

  useEffect(() => {
    setMounted(true);
  }, []);

  useEffect(() => {
    if (!open) return;

    const updatePosition = () => {
      const rect = anchorRef.current?.getBoundingClientRect();
      if (!rect) {
        setPanelStyle({
          top: 40,
          right: VIEWPORT_PAD,
          visibility: "visible",
        });
        return;
      }

      if (placement === "right") {
        const left = Math.min(rect.right + VIEWPORT_PAD, window.innerWidth - PANEL_WIDTH - VIEWPORT_PAD);
        const top = Math.min(
          Math.max(VIEWPORT_PAD, rect.top),
          window.innerHeight - PANEL_MAX_HEIGHT - VIEWPORT_PAD,
        );
        setPanelStyle({ top, left, visibility: "visible" });
        return;
      }

      const top = rect.bottom + VIEWPORT_PAD;
      const left = Math.min(
        Math.max(VIEWPORT_PAD, rect.right - PANEL_WIDTH),
        window.innerWidth - PANEL_WIDTH - VIEWPORT_PAD,
      );
      setPanelStyle({ top, left, visibility: "visible" });
    };

    updatePosition();
    window.addEventListener("resize", updatePosition);
    window.addEventListener("scroll", updatePosition, true);
    return () => {
      window.removeEventListener("resize", updatePosition);
      window.removeEventListener("scroll", updatePosition, true);
    };
  }, [open, anchorRef, placement, jobs.length]);

  useEffect(() => {
    if (!open) return;
    const onPointerDown = (event: MouseEvent) => {
      const target = event.target as Node;
      if (panelRef.current?.contains(target)) return;
      if (anchorRef.current?.contains(target)) return;
      onOpenChange(false);
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") onOpenChange(false);
    };
    document.addEventListener("mousedown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("mousedown", onPointerDown);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [open, onOpenChange, anchorRef]);

  const finishedCount = jobs.length - active.length;

  const panel = open && mounted ? (
    <div
      ref={panelRef}
      role="dialog"
      aria-label="Downloads"
      className="fixed w-[min(25rem,calc(100vw-1rem))] overflow-hidden rounded-xl border border-border bg-panel-secondary shadow-sm"
      style={{ ...panelStyle, zIndex: 500 }}
      data-theme-slot="shell.downloadPopup"
    >
      <div className="flex items-center justify-between px-2 py-1.5">
        <div className="flex items-center gap-2">
          <span className="text-sm text-text-primary">Downloads</span>
          {activeCount > 0 && (
            <span className="rounded bg-panel-hover px-1.5 py-0.5 text-sm text-text-muted">
              {activeCount} Active
            </span>
          )}
        </div>
        <div className="flex items-center gap-1">
          {finishedCount > 0 && (
            <button
              type="button"
              className="text-sm text-error hover:underline disabled:opacity-50"
              disabled={clearing}
              onClick={() => void clearFinished()}
            >
              {clearing ? "clearing…" : "× clear"}
            </button>
          )}
          <button
            type="button"
            className="rounded p-1 text-text-muted hover:bg-panel-hover hover:text-text-primary"
            aria-label="Close downloads"
            onClick={() => onOpenChange(false)}
          >
            <X size={14} />
          </button>
        </div>
      </div>

      {hasQueuedManual && (
        <div className="px-3 py-2">
          <Button
            variant="secondary"
            size="sm"
            className="h-7 w-full text-xs"
            disabled={startingPending}
            onClick={() => void startQueued()}
          >
            {startingPending ? "Starting…" : "Start pending downloads"}
          </Button>
        </div>
      )}

      {jobs.length === 0 ? (
        <p className="px-3 py-6 text-center text-sm text-text-muted">
          {loading ? "Loading…" : "No downloads in queue"}
        </p>
      ) : (
        <ul className="max-h-80 overflow-y-auto">
          {jobs.map((job) => {
            const image = downloadJobImage(job);
            const activeJob = isActiveDownload(job);
            return (
              <li key={job.id} className="border-b border-border px-3 py-2.5 last:border-b-0">
                <div className="flex items-start gap-2.5">
                  <div className="mt-0.5 h-9 w-9 shrink-0 overflow-hidden rounded-md bg-panel-hover">
                    {image ? (
                      <img src={image} alt="" className="h-full w-full object-cover" />
                    ) : (
                      <div className="flex h-full w-full items-center justify-center text-sm text-text-muted">
                        mod
                      </div>
                    )}
                  </div>
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm text-text-primary">{job.modName}</p>
                    {activeJob && job.progress > 0 && (
                      <div className="mt-1.5 flex gap-0.5">
                        {Array.from({ length: 4 }).map((_, i) => (
                          <div
                            key={i}
                            className={`h-1 flex-1 rounded-full ${
                              job.progress >= (i + 1) * 25 ? "bg-primary" : "bg-panel-hover"
                            }`}
                          />
                        ))}
                      </div>
                    )}
                    <div className="mt-1 flex items-center gap-1.5 text-sm text-text-muted">
                      {activeJob && <Loader2 size={10} className="animate-spin" />}
                      <span>{statusLabel(job.status)}</span>
                      {activeJob && job.progress > 0 && <span>· {job.progress}%</span>}
                    </div>
                    {job.error && <p className="mt-0.5 text-sm text-error">{job.error}</p>}
                  </div>
                  {activeJob && (
                    <button
                      type="button"
                      className="shrink-0 text-text-muted hover:text-error"
                      aria-label="Cancel download"
                      onClick={() => void cancelJob(job.id)}
                    >
                      <X size={14} />
                    </button>
                  )}
                </div>
              </li>
            );
          })}
        </ul>
      )}

      <div className="flex items-center justify-between px-2 py-1.5">
        {activeCount > 0 ? (
          <span className="text-sm text-text-muted">{overallProgress}% Overall</span>
        ) : (
          <span className="text-sm text-text-muted">Queue idle</span>
        )}
        <div className="flex items-center gap-2">
          <button
            type="button"
            className="text-sm text-text-muted hover:text-text-primary"
            onClick={refresh}
          >
            Refresh
          </button>
          {onViewAll && (
            <button
              type="button"
              className="text-sm text-text-primary hover:underline"
              onClick={() => {
                onOpenChange(false);
                onViewAll();
              }}
            >
              View All
            </button>
          )}
        </div>
      </div>
    </div>
  ) : null;

  return (
    <>
      {children}
      {mounted && panel ? createPortal(panel, document.body) : null}
    </>
  );
}
