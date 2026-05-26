/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useCallback, useLayoutEffect, useMemo, useRef, useState } from "react";
import { LoadoutPicker } from "@/features/mod-tools/ui/loadout-picker";
import { LibraryMenu } from "@/features/mod-tools/ui/library-menu";
import type { DetectedGame, Loadout, ModTableColumn } from "@/types";
import {
  ArrowUpCircle,
  ChevronDown,
  FolderOpen,
  GitBranch,
  History,
  Link2,
  MoreHorizontal,
  PlusSquare,
  RefreshCw,
  RotateCcw,
  Search,
  Tag,
  Unlink2,
} from "lucide-react";
import {
  Dropdown,
  DropdownContent,
  DropdownItem,
  DropdownSub,
  DropdownSubContent,
  DropdownSubTrigger,
  DropdownTrigger,
} from "@/components/ui/dropdown";

const OVERFLOW_BUTTON_WIDTH = 36;

type Props = {
  game: DetectedGame;
  gameId: string;
  loadoutId?: string;
  loadoutName?: string;
  deployPathOverride?: string;
  driftDetected?: boolean;
  onDeployPathOverride: (path: string | undefined) => void;
  onPurgeComplete: () => void;
  onDriftChecked: (drift: boolean) => void;
  onLoadoutChange: (loadout: Loadout) => void;
  onAdd: () => void;
  onRefresh: () => void;
  onDeploy: () => void;
  onPurge: () => void;
  onResetManifest?: () => void;
  deploying: boolean;
  purging: boolean;
  refreshing: boolean;
  searchOpen: boolean;
  onSearchToggle: () => void;
  searchQuery: string;
  onSearchChange: (q: string) => void;
  columns: ModTableColumn[];
  onToggleColumn: (id: string) => void;
  onShowCategories?: () => void;
  onManageRules?: () => void;
  onCheckUpdates?: () => void;
  onDownloadUpdates?: () => void;
  checkingUpdates?: boolean;
  updateCount?: number;
  onOpenStaging?: () => void;
  onOpenGameFolder?: () => void;
  onShowHistory?: () => void;
};

type ToolbarEntry = {
  id: string;
  node: React.ReactNode;
  menu?: React.ReactNode;
};

export function ModToolbar({
  game,
  gameId,
  loadoutId,
  loadoutName,
  deployPathOverride,
  driftDetected,
  onDeployPathOverride,
  onPurgeComplete,
  onDriftChecked,
  onLoadoutChange,
  onAdd,
  onRefresh,
  onDeploy,
  onPurge,
  onResetManifest,
  deploying,
  purging,
  refreshing,
  searchOpen,
  onSearchToggle,
  searchQuery,
  onSearchChange,
  columns,
  onToggleColumn,
  onShowCategories,
  onManageRules,
  onCheckUpdates,
  onDownloadUpdates,
  checkingUpdates,
  updateCount,
  onOpenStaging,
  onOpenGameFolder,
  onShowHistory,
}: Props) {
  const primaryEntries = useMemo(
    () =>
      buildPrimaryEntries({
        onAdd,
        onCheckUpdates,
        onDownloadUpdates,
        checkingUpdates,
        updateCount,
        onShowCategories,
        onManageRules,
        onDeploy,
        deploying,
        onPurge,
        purging,
        onResetManifest,
        onShowHistory,
        onOpenStaging,
        onOpenGameFolder,
      }),
    [
      onAdd,
      onCheckUpdates,
      onDownloadUpdates,
      checkingUpdates,
      updateCount,
      onShowCategories,
      onManageRules,
      onDeploy,
      deploying,
      onPurge,
      purging,
      onResetManifest,
      onShowHistory,
      onOpenStaging,
      onOpenGameFolder,
    ],
  );

  const containerRef = useRef<HTMLDivElement>(null);
  const rightRef = useRef<HTMLDivElement>(null);
  const measureRefs = useRef<(HTMLDivElement | null)[]>([]);
  const [visibleCount, setVisibleCount] = useState(primaryEntries.length);

  const measure = useCallback(() => {
    const container = containerRef.current;
    const right = rightRef.current;
    if (!container || !right) return;

    const rightWidth = right.offsetWidth;
    const widths = primaryEntries.map(
      (_, i) => measureRefs.current[i]?.offsetWidth ?? 0,
    );
    let available = container.clientWidth - rightWidth;
    let count = 0;

    for (let i = 0; i < primaryEntries.length; i++) {
      const w = widths[i] ?? 0;
      const remaining = primaryEntries.length - (i + 1);
      const reserveOverflow = remaining > 0 ? OVERFLOW_BUTTON_WIDTH : 0;
      if (available >= w + reserveOverflow) {
        available -= w;
        count = i + 1;
      } else {
        break;
      }
    }

    if (count === 0 && primaryEntries.length > 0) {
      count = 1;
    }

    setVisibleCount((prev) => (prev === count ? prev : count));
  }, [primaryEntries]);

  useLayoutEffect(() => {
    measure();
    const container = containerRef.current;
    if (!container) return;
    const ro = new ResizeObserver(() => measure());
    ro.observe(container);
    if (rightRef.current) ro.observe(rightRef.current);
    return () => ro.disconnect();
  }, [measure]);

  const visible = primaryEntries.slice(0, visibleCount);
  const overflow = primaryEntries.slice(visibleCount);

  return (
    <div className="relative flex shrink-0 flex-col border-b border-border">
      <div
        ref={containerRef}
        className="flex h-9 min-w-0 items-center bg-toolbar text-text-primary"
      >
        <div className="flex min-w-0 flex-1 items-center overflow-hidden">
          {visible.map((entry) => (
            <div key={entry.id} className="shrink-0">
              {entry.node}
            </div>
          ))}
          {overflow.length > 0 && (
            <Dropdown>
              <DropdownTrigger asChild>
                <button
                  type="button"
                  title="More actions"
                  className="flex h-8 w-9 shrink-0 items-center justify-center rounded-lg hover:bg-white/10"
                >
                  <MoreHorizontal size={18} />
                </button>
              </DropdownTrigger>
              <DropdownContent align="start" className="min-w-40">
                {overflow.map(
                  (entry) =>
                    entry.menu && <div key={entry.id}>{entry.menu}</div>,
                )}
              </DropdownContent>
            </Dropdown>
          )}
        </div>

        <div
          ref={rightRef}
          className="ml-auto flex shrink-0 items-center gap-1 px-2"
        >
          <ToolbarAction
            title="Reload library"
            onClick={onRefresh}
            disabled={refreshing}
            compact
          >
            <RefreshCw size={16} className={refreshing ? "animate-spin" : ""} />
          </ToolbarAction>
          <ToolbarAction
            title="Search mods"
            onClick={onSearchToggle}
            active={searchOpen}
            compact
          >
            <Search size={16} />
          </ToolbarAction>
          <LoadoutPicker
            gameId={gameId}
            activeLoadoutId={loadoutId}
            loadoutName={loadoutName}
            onLoadoutChange={onLoadoutChange}
          />
          <LibraryMenu
            game={game}
            deployPathOverride={deployPathOverride}
            driftDetected={driftDetected}
            onDeployPathOverride={onDeployPathOverride}
            onPurgeComplete={onPurgeComplete}
            onDriftChecked={onDriftChecked}
          />
        </div>
      </div>

      <div
        className="pointer-events-none absolute left-0 top-0 -z-10 flex h-0 overflow-hidden opacity-0"
        aria-hidden
      >
        {primaryEntries.map((entry, i) => (
          <div
            key={entry.id}
            ref={(el) => {
              measureRefs.current[i] = el;
            }}
            className="shrink-0"
          >
            {entry.node}
          </div>
        ))}
      </div>

      {searchOpen && (
        <div className="flex items-center gap-2 border-b border-border bg-panel-secondary px-3 py-2">
          <Search size={14} className="text-text-muted" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            placeholder="Filter mods…"
            className="min-w-0 flex-1 bg-transparent text-md text-text-primary outline-none"
          />
          <Dropdown>
            <DropdownTrigger asChild>
              <button
                type="button"
                className="text-sm text-text-muted hover:text-text-primary"
              >
                Columns
              </button>
            </DropdownTrigger>
            <DropdownContent align="end" className="w-44">
              {columns.map((col) => (
                <DropdownItem
                  key={col.id}
                  onClick={() => onToggleColumn(col.id)}
                >
                  {col.visible ? "☑" : "☐"} {col.label}
                </DropdownItem>
              ))}
            </DropdownContent>
          </Dropdown>
        </div>
      )}
    </div>
  );
}

function buildPrimaryEntries(opts: {
  onAdd: () => void;
  onCheckUpdates?: () => void;
  onDownloadUpdates?: () => void;
  checkingUpdates?: boolean;
  updateCount?: number;
  onShowCategories?: () => void;
  onManageRules?: () => void;
  onDeploy: () => void;
  deploying: boolean;
  onPurge: () => void;
  purging: boolean;
  onResetManifest?: () => void;
  onShowHistory?: () => void;
  onOpenStaging?: () => void;
  onOpenGameFolder?: () => void;
}): ToolbarEntry[] {
  const entries: ToolbarEntry[] = [
    {
      id: "install",
      node: (
        <ToolbarAction title="Install from file" label="Install" onClick={opts.onAdd}>
          <PlusSquare size={16} />
        </ToolbarAction>
      ),
      menu: <DropdownItem onClick={opts.onAdd}>Install from file</DropdownItem>,
    },
  ];

  if (opts.onCheckUpdates) {
    entries.push({
      id: "updates",
      node: (
        <Dropdown>
          <DropdownTrigger asChild>
            <button
              type="button"
              className="flex h-8 items-center gap-1 rounded-lg px-3 text-sm hover:bg-white/10 disabled:opacity-50"
              disabled={opts.checkingUpdates}
            >
              <ArrowUpCircle
                size={16}
                className={opts.checkingUpdates ? "animate-spin" : ""}
              />
              <span className="hidden sm:inline">Check for Updates</span>
              <ChevronDown size={12} className="opacity-70" />
            </button>
          </DropdownTrigger>
          <DropdownContent align="start">
            <DropdownItem onClick={opts.onCheckUpdates}>Check now</DropdownItem>
            {opts.onDownloadUpdates && opts.updateCount ? (
              <DropdownItem onClick={opts.onDownloadUpdates}>
                Download {opts.updateCount} update(s)
              </DropdownItem>
            ) : null}
            <DropdownItem disabled={!opts.updateCount}>
              {opts.updateCount
                ? `${opts.updateCount} update(s) available`
                : "No updates found"}
            </DropdownItem>
          </DropdownContent>
        </Dropdown>
      ),
      menu: (
        <>
          <DropdownItem onClick={opts.onCheckUpdates} disabled={opts.checkingUpdates}>
            Check for updates
          </DropdownItem>
          {opts.onDownloadUpdates && opts.updateCount ? (
            <DropdownItem onClick={opts.onDownloadUpdates}>
              Download {opts.updateCount} update(s)
            </DropdownItem>
          ) : null}
        </>
      ),
    });
  }

  entries.push(
    {
      id: "categories",
      node: (
        <ToolbarAction
          title="Filter by category"
          label="Categories"
          onClick={opts.onShowCategories}
        >
          <Tag size={16} />
        </ToolbarAction>
      ),
      menu: (
        <DropdownItem onClick={opts.onShowCategories}>Categories</DropdownItem>
      ),
    },
    {
      id: "rules",
      node: (
        <ToolbarAction
          title="Manage conflict rules"
          label="Manage Rules"
          onClick={opts.onManageRules}
        >
          <GitBranch size={16} />
        </ToolbarAction>
      ),
      menu: (
        <DropdownItem onClick={opts.onManageRules}>Manage Rules</DropdownItem>
      ),
    },
    {
      id: "deploy",
      node: (
        <ToolbarAction
          title="Deploy mods to game folder"
          label="Deploy"
          onClick={opts.onDeploy}
          disabled={opts.deploying}
        >
          <Link2 size={16} />
          {opts.deploying && <span className="ml-1 text-sm">…</span>}
        </ToolbarAction>
      ),
      menu: (
        <DropdownItem onClick={opts.onDeploy} disabled={opts.deploying}>
          Deploy
        </DropdownItem>
      ),
    },
    {
      id: "purge",
      node: (
        <ToolbarAction
          title="Purge deployed files"
          label="Purge"
          onClick={opts.onPurge}
          disabled={opts.purging}
        >
          <Unlink2 size={16} />
        </ToolbarAction>
      ),
      menu: (
        <DropdownItem onClick={opts.onPurge} disabled={opts.purging}>
          Purge
        </DropdownItem>
      ),
    },
    {
      id: "reset",
      node: (
        <ToolbarAction
          title="Reset loadout to last deploy"
          label="Reset"
          onClick={opts.onResetManifest}
        >
          <RotateCcw size={16} />
        </ToolbarAction>
      ),
      menu: (
        <DropdownItem onClick={opts.onResetManifest}>Reset loadout</DropdownItem>
      ),
    },
    {
      id: "history",
      node: (
        <ToolbarAction title="Deploy history" label="History" onClick={opts.onShowHistory}>
          <History size={16} />
        </ToolbarAction>
      ),
      menu: <DropdownItem onClick={opts.onShowHistory}>Deploy history</DropdownItem>,
    },
    {
      id: "open",
      node: (
        <Dropdown>
          <DropdownTrigger asChild>
            <button
              type="button"
              className="flex h-8 items-center gap-1 rounded-lg px-3 text-sm hover:bg-white/10"
            >
              <FolderOpen size={16} />
              <span className="hidden sm:inline">Open</span>
              <ChevronDown size={12} className="opacity-70" />
            </button>
          </DropdownTrigger>
          <DropdownContent align="start">
            <DropdownItem disabled={!opts.onOpenStaging} onClick={opts.onOpenStaging}>
              Staging folder
            </DropdownItem>
            <DropdownItem
              disabled={!opts.onOpenGameFolder}
              onClick={opts.onOpenGameFolder}
            >
              Game install folder
            </DropdownItem>
          </DropdownContent>
        </Dropdown>
      ),
      menu: (
        <DropdownSub>
          <DropdownSubTrigger>Open folder</DropdownSubTrigger>
          <DropdownSubContent>
            <DropdownItem disabled={!opts.onOpenStaging} onClick={opts.onOpenStaging}>
              Staging folder
            </DropdownItem>
            <DropdownItem
              disabled={!opts.onOpenGameFolder}
              onClick={opts.onOpenGameFolder}
            >
              Game install folder
            </DropdownItem>
          </DropdownSubContent>
        </DropdownSub>
      ),
    },
  );

  return entries;
}

function ToolbarAction({
  children,
  label,
  title,
  onClick,
  disabled,
  active,
  compact,
}: {
  children: React.ReactNode;
  label?: string;
  title: string;
  onClick?: () => void;
  disabled?: boolean;
  active?: boolean;
  compact?: boolean;
}) {
  return (
    <button
      type="button"
      title={title}
      disabled={disabled}
      onClick={onClick}
      className={`flex h-8 items-center gap-1.5 rounded-lg hover:bg-white/10 disabled:opacity-40 ${
        compact ? "px-2" : "px-3"
      } ${active ? "bg-white/15" : ""}`}
    >
      {children}
      {label && !compact && (
        <span className="hidden text-sm md:inline">{label}</span>
      )}
    </button>
  );
}
