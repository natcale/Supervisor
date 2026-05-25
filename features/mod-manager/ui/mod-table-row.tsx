/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { Button } from "@/components/ui/button";
import { EnabledCell } from "@/features/mod-manager/ui/enabled-cell";
import type { ModManifest, ModTableColumn } from "@/types";
import { AlertTriangle, ChevronDown, GripVertical } from "lucide-react";
import {
  Dropdown,
  DropdownContent,
  DropdownItem,
  DropdownTrigger,
} from "@/components/ui/dropdown";

type Props = {
  mod: ModManifest;
  enabled: boolean;
  selected: boolean;
  columns: ModTableColumn[];
  hasConflict?: boolean;
  draggable?: boolean;
  dragging?: boolean;
  onDragStart?: () => void;
  onDragOver?: (e: React.DragEvent) => void;
  onDrop?: () => void;
  onDragEnd?: () => void;
  onSelect: (modId: string, additive: boolean, range: boolean) => void;
  onToggle: (modId: string) => void;
  onRemove: (modId: string) => void;
  onConfigureFomod?: (mod: ModManifest) => void;
  onShowInfo?: (mod: ModManifest) => void;
  onReinstall?: (mod: ModManifest) => void;
  onOpenFolder?: (mod: ModManifest) => void;
};

export function ModTableRow({
  mod,
  enabled,
  selected,
  columns,
  hasConflict,
  draggable,
  dragging,
  onDragStart,
  onDragOver,
  onDrop,
  onDragEnd,
  onSelect,
  onToggle,
  onRemove,
  onConfigureFomod,
  onShowInfo,
  onReinstall,
  onOpenFolder,
}: Props) {
  const pending = mod.installState === "pendingFomod" || mod.needsFomod;
  const visible = (id: string) => columns.find((c) => c.id === id)?.visible !== false;

  return (
    <tr
      draggable={draggable}
      onDragStart={onDragStart}
      onDragOver={onDragOver}
      onDrop={onDrop}
      onDragEnd={onDragEnd}
      className={`hover:bg-table-row-hover ${
        selected ? "bg-panel-hover" : ""
      } ${dragging ? "opacity-50" : ""}`}
      onClick={(e) => onSelect(mod.id, e.ctrlKey || e.metaKey, e.shiftKey)}
    >
      {draggable && (
        <td className="w-8 px-1 py-1.5 text-text-muted">
          <GripVertical size={14} className="cursor-grab" />
        </td>
      )}
      {visible("enabled") && (
        <td className="px-2 py-1.5">
          <EnabledCell
            enabled={enabled}
            disabled={pending}
            onToggle={() => onToggle(mod.id)}
          />
        </td>
      )}
      {visible("name") && (
        <td className="max-w-0 px-2 py-1.5">
          <div className="flex min-w-0 items-center gap-2">
            {mod.nexus?.pictureUrl && (
              <img
                src={mod.nexus.pictureUrl}
                alt=""
                className="h-8 w-8 shrink-0 object-cover"
              />
            )}
            <div className="min-w-0">
              <button
                type="button"
                className="truncate text-left text-sm text-text-primary hover:underline"
                onClick={(e) => {
                  e.stopPropagation();
                  onShowInfo?.(mod);
                }}
              >
                {mod.name}
              </button>
              {pending && (
                <p className="text-sm text-primary">FOMOD — configure</p>
              )}
              {hasConflict && (
                <AlertTriangle size={12} className="inline text-[var(--warning)]" />
              )}
            </div>
          </div>
        </td>
      )}
      {visible("version") && (
        <td className="px-2 py-1.5 text-xs text-text-secondary">
          <span>{mod.nexus?.version ?? "—"}</span>
          {mod.nexus?.updateAvailable && (
            <span className="ml-1 text-[var(--warning)]" title={`Update: ${mod.nexus.latestVersion ?? "available"}`}>
              ↑
            </span>
          )}
        </td>
      )}
      {visible("author") && (
        <td className="truncate px-2 py-1.5 text-xs text-text-secondary">
          {mod.nexus?.author ?? "—"}
        </td>
      )}
      {visible("category") && (
        <td className="truncate px-2 py-1.5 text-xs text-text-secondary">
          {mod.nexus?.category ?? (mod.nexus ? "Uncategorized" : "Local")}
        </td>
      )}
      {visible("actions") && (
        <td className="px-2 py-1.5">
          <div className="flex items-center gap-1">
            {pending && mod.slug && onConfigureFomod && (
              <Button
                variant="secondary"
                size="sm"
                className="h-7 text-xs"
                onClick={(e) => {
                  e.stopPropagation();
                  onConfigureFomod(mod);
                }}
              >
                Configure
              </Button>
            )}
            <Dropdown>
              <DropdownTrigger asChild>
                <button
                  type="button"
                  className="inline-flex h-7 items-center gap-0.5 bg-primary px-2 text-sm text-white hover:bg-primary-hover"
                  onClick={(e) => e.stopPropagation()}
                >
                  Remove
                  <ChevronDown size={16} />
                </button>
              </DropdownTrigger>
              <DropdownContent align="end">
                <DropdownItem
                  onClick={() => onRemove(mod.id)}
                  className="text-[var(--error)]"
                >
                  Remove from library
                </DropdownItem>
                {mod.slug && onOpenFolder && (
                  <DropdownItem onClick={() => onOpenFolder(mod)}>Open folder</DropdownItem>
                )}
                {onReinstall && (
                  <DropdownItem onClick={() => onReinstall(mod)}>Rescan from staging</DropdownItem>
                )}
                {pending && mod.slug && (
                  <DropdownItem onClick={() => onConfigureFomod?.(mod)}>
                    Configure FOMOD
                  </DropdownItem>
                )}
              </DropdownContent>
            </Dropdown>
          </div>
        </td>
      )}
    </tr>
  );
}
