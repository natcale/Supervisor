/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Image from "next/image";
import { Download, Settings } from "lucide-react";
import type { ShellView } from "@/types";
import { DownloadPopup } from "@/features/downloads/ui/download-popup";
import { useDownloadQueue } from "@/features/downloads/model/use-download-queue";

function MinimizeIcon() {
  return (
    <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden>
      <path d="M0 5H10" stroke="currentColor" strokeWidth="1" fill="none" />
    </svg>
  );
}

function MaximizeIcon() {
  return (
    <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden>
      <rect x="0.5" y="0.5" width="9" height="9" stroke="currentColor" strokeWidth="1" fill="none" />
    </svg>
  );
}

function RestoreIcon() {
  return (
    <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden>
      <path d="M2.5 2.5V0.5H9.5V7.5H7.5" stroke="currentColor" strokeWidth="1" fill="none" />
      <rect x="0.5" y="2.5" width="7" height="7" stroke="currentColor" strokeWidth="1" fill="none" />
    </svg>
  );
}

function CloseIcon() {
  return (
    <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden>
      <path d="M0.5 0.5L9.5 9.5M9.5 0.5L0.5 9.5" stroke="currentColor" strokeWidth="1" fill="none" />
    </svg>
  );
}

type TitlebarProps = {
  /** Fixed-size windows (onboarding): no maximize state tracking. */
  variant?: "default" | "compact";
  onNavigate?: (view: ShellView) => void;
};

export function Titlebar({
  variant = "default",
  onNavigate,
}: TitlebarProps) {
  const compact = variant === "compact";
  const [maximized, setMaximized] = useState(false);
  const downloadAnchorRef = useRef<HTMLButtonElement>(null);
  const [downloadsOpen, setDownloadsOpen] = useState(false);
  const { activeCount, overallProgress } = useDownloadQueue();

  useEffect(() => {
    if (compact) return;
    const win = getCurrentWindow();
    void win.isMaximized().then(setMaximized);
    let unlisten: (() => void) | undefined;
    void win.onResized(async () => {
      setMaximized(await win.isMaximized());
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, [compact]);

  const minimize = () => getCurrentWindow().minimize();
  const toggleMaximize = async () => {
    const win = getCurrentWindow();
    if (await win.isMaximized()) await win.unmaximize();
    else await win.maximize();
  };
  const close = () => getCurrentWindow().close();

  return (
    <header className="titlebar relative z-[300] flex h-[33px] shrink-0 items-center justify-between text-sm select-none" data-theme-slot="shell.titlebar">
      <div className="titlebar-left z-20 flex h-full items-center gap-2 px-3">
        <Image src="/logo.svg" alt="" width={16} height={16} className="h-4 w-auto rounded-sm" style={{ width: "auto", height: "16px" }} unoptimized />
        <span className="text-xs text-text-secondary">Supervisor</span>
      </div>

      <div className="titlebar-drag absolute inset-x-0 top-0 h-[33px]" data-tauri-drag-region />

      <div className="titlebar-right z-20 flex h-full items-center">
        {!compact && onNavigate && (
          <>
            <DownloadPopup
              open={downloadsOpen}
              onOpenChange={setDownloadsOpen}
              anchorRef={downloadAnchorRef}
              onViewAll={() => onNavigate("downloads")}
            >
              <button
                ref={downloadAnchorRef}
                type="button"
                title="Downloads"
                className="titlebar-control relative flex h-full w-[46px] items-center justify-center hover:bg-panel-hover"
                onClick={() => setDownloadsOpen((v) => !v)}
              >
                {activeCount > 0 && (
                  <svg
                    className="pointer-events-none absolute inset-2 h-[calc(100%-16px)] w-[calc(100%-16px)] -rotate-90"
                    viewBox="0 0 48 48"
                    aria-hidden
                  >
                    <circle cx="24" cy="24" r="22" fill="none" stroke="var(--border)" strokeWidth="2" />
                    <circle
                      cx="24"
                      cy="24"
                      r="22"
                      fill="none"
                      stroke="var(--primary)"
                      strokeWidth="2"
                      strokeDasharray={`${(overallProgress / 100) * 138} 138`}
                      strokeLinecap="round"
                    />
                  </svg>
                )}
                <Download size={14} />
                {activeCount > 0 && (
                  <span className="absolute right-2 top-1.5 flex h-3.5 min-w-3.5 items-center justify-center rounded-full bg-primary px-0.5 text-[9px] text-white">
                    {activeCount}
                  </span>
                )}
              </button>
            </DownloadPopup>
            <button
              type="button"
              title="Settings"
              className="titlebar-control flex h-full w-[46px] items-center justify-center hover:bg-panel-hover"
              onClick={() => onNavigate("settings")}
            >
              <Settings size={14} />
            </button>
          </>
        )}
        <button
          type="button"
          className="titlebar-control flex h-full w-[46px] items-center justify-center hover:bg-panel-hover"
          onClick={minimize}
        >
          <MinimizeIcon />
        </button>
        {!compact && (
          <button
            type="button"
            className="titlebar-control flex h-full w-[46px] items-center justify-center hover:bg-panel-hover"
            onClick={() => void toggleMaximize()}
          >
            {maximized ? <RestoreIcon /> : <MaximizeIcon />}
          </button>
        )}
        <button
          type="button"
          className="titlebar-control close flex h-full w-[46px] items-center justify-center hover:bg-[#e81123] hover:text-white"
          onClick={close}
        >
          <CloseIcon />
        </button>
      </div>
    </header>
  );
}
