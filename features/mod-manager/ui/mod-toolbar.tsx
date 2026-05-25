/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

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
  DropdownTrigger,
} from "@/components/ui/dropdown";

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
  return (
    <div className="flex shrink-0 flex-col border-b border-border">
      <div className="flex flex-wrap items-stretch bg-toolbar text-white">
        <ToolbarAction title="Install from file" label="Install" onClick={onAdd}>
          <PlusSquare size={16} />
        </ToolbarAction>

        {onCheckUpdates && (
          <Dropdown>
            <DropdownTrigger asChild>
              <button
                type="button"
                className="flex h-9 items-center gap-1 border-r border-white/10 px-3 text-xs hover:bg-white/10 disabled:opacity-50"
                disabled={checkingUpdates}
              >
                <ArrowUpCircle size={16} className={checkingUpdates ? "animate-spin" : ""} />
                <span className="hidden sm:inline">Check for Updates</span>
                <ChevronDown size={12} className="opacity-70" />
              </button>
            </DropdownTrigger>
            <DropdownContent align="start">
              <DropdownItem onClick={onCheckUpdates}>Check now</DropdownItem>
              {onDownloadUpdates && updateCount ? (
                <DropdownItem onClick={onDownloadUpdates}>
                  Download {updateCount} update(s)
                </DropdownItem>
              ) : null}
              <DropdownItem disabled={!updateCount}>
                {updateCount ? `${updateCount} update(s) available` : "No updates found"}
              </DropdownItem>
            </DropdownContent>
          </Dropdown>
        )}

        <ToolbarAction title="Filter by category" label="Categories" onClick={onShowCategories}>
          <Tag size={16} />
        </ToolbarAction>

        <ToolbarAction title="Manage conflict rules" label="Manage Rules" onClick={onManageRules}>
          <GitBranch size={16} />
        </ToolbarAction>

        <ToolbarAction title="Deploy mods to game folder" label="Deploy" onClick={onDeploy} disabled={deploying}>
          <Link2 size={16} />
          {deploying && <span className="ml-1 text-[10px]">…</span>}
        </ToolbarAction>

        <ToolbarAction title="Purge deployed files" label="Purge" onClick={onPurge} disabled={purging}>
          <Unlink2 size={16} />
        </ToolbarAction>

        <ToolbarAction title="Reset loadout to last deploy" label="Reset" onClick={onResetManifest}>
          <RotateCcw size={16} />
        </ToolbarAction>

        <ToolbarAction title="Deploy history" label="History" onClick={onShowHistory}>
          <History size={16} />
        </ToolbarAction>

        <Dropdown>
          <DropdownTrigger asChild>
            <button
              type="button"
              className="flex h-9 items-center gap-1 border-r border-white/10 px-3 text-xs hover:bg-white/10"
            >
              <FolderOpen size={16} />
              <span className="hidden sm:inline">Open</span>
              <ChevronDown size={12} className="opacity-70" />
            </button>
          </DropdownTrigger>
          <DropdownContent align="start">
            <DropdownItem disabled={!onOpenStaging} onClick={onOpenStaging}>
              Staging folder
            </DropdownItem>
            <DropdownItem disabled={!onOpenGameFolder} onClick={onOpenGameFolder}>
              Game install folder
            </DropdownItem>
          </DropdownContent>
        </Dropdown>

        <div className="ml-auto flex items-center gap-1 px-2">
          <ToolbarAction title="Reload library" onClick={onRefresh} disabled={refreshing} compact>
            <RefreshCw size={16} className={refreshing ? "animate-spin" : ""} />
          </ToolbarAction>
          <ToolbarAction title="Search mods" onClick={onSearchToggle} active={searchOpen} compact>
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

      {searchOpen && (
        <div className="flex items-center gap-2 border-b border-border bg-panel-secondary px-3 py-1.5">
          <Search size={14} className="text-text-muted" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            placeholder="Filter mods…"
            className="min-w-0 flex-1 bg-transparent text-sm text-text-primary outline-none"
          />
          <Dropdown>
            <DropdownTrigger asChild>
              <button type="button" className="text-xs text-text-muted hover:text-text-primary">
                Columns
              </button>
            </DropdownTrigger>
            <DropdownContent align="end" className="w-44">
              {columns.map((col) => (
                <DropdownItem key={col.id} onClick={() => onToggleColumn(col.id)}>
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
      className={`flex h-9 items-center gap-1.5 border-r border-white/10 hover:bg-white/10 disabled:opacity-40 ${
        compact ? "px-2" : "px-3"
      } ${active ? "bg-white/15" : ""}`}
    >
      {children}
      {label && !compact && <span className="hidden text-xs md:inline">{label}</span>}
    </button>
  );
}
