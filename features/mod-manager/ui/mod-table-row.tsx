/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useRef } from "react";
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
  rowIndex: number;
  enabled: boolean;
  selected: boolean;
  columns: ModTableColumn[];
  hasConflict?: boolean;
  reorderable?: boolean;
  dragging?: boolean;
  dropTarget?: boolean;
  onGripPointerDown?: (index: number) => void;
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
  rowIndex,
  enabled,
  selected,
  columns,
  hasConflict,
  reorderable,
  dragging,
  dropTarget,
  onGripPointerDown,
  onSelect,
  onToggle,
  onRemove,
  onConfigureFomod,
  onShowInfo,
  onReinstall,
  onOpenFolder,
}: Props) {
  const pending = mod.installState === "pendingFomod" || mod.needsFomod;
  const visible = (id: string) =>
    columns.find((c) => c.id === id)?.visible !== false;
  const didDragRef = useRef(false);
  const author = mod.nexus?.author ?? "—";
  const version = mod.nexus?.version ?? "—";
  const category = mod.nexus?.category ?? (mod.nexus ? "Uncategorized" : "Local");

  const handleRowClick = (e: React.MouseEvent) => {
    if (didDragRef.current) {
      didDragRef.current = false;
      return;
    }
    onSelect(mod.id, e.ctrlKey || e.metaKey, e.shiftKey);
  };

  return (
    <tr
      data-mod-row-index={rowIndex}
      onClick={handleRowClick}
      className={`cursor-default hover:bg-table-row-hover ${
        selected ? "bg-panel-hover" : ""
      } ${dragging ? "opacity-50" : ""} ${
        dropTarget ? "ring-1 ring-inset ring-primary/50" : ""
      } ${reorderable && dragging ? "select-none" : ""}`}
    >
      {reorderable && (
        <td className="w-8 px-1 py-1.5 text-text-muted">
          <button
            type="button"
            title="Drag to reorder load order"
            className="inline-flex cursor-grab touch-none active:cursor-grabbing"
            onPointerDown={(e) => {
              if (e.button !== 0) return;
              e.preventDefault();
              e.stopPropagation();
              didDragRef.current = true;
              (e.currentTarget as HTMLButtonElement).setPointerCapture(e.pointerId);
              onGripPointerDown?.(rowIndex);
            }}
          >
            <GripVertical size={14} />
          </button>
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
        <td className="max-w-0 overflow-hidden px-2 py-1.5">
          <div className="flex min-w-0 items-center gap-2 overflow-hidden">
            {mod.nexus?.pictureUrl && (
              <img
                src={mod.nexus.pictureUrl}
                alt=""
                className="h-8 w-8 shrink-0 object-cover"
              />
            )}
            <div className="min-w-0 flex-1 overflow-hidden">
              <button
                type="button"
                className="block w-full truncate text-left text-sm text-text-primary hover:underline"
                title={mod.name}
                onClick={(e) => {
                  e.stopPropagation();
                  onShowInfo?.(mod);
                }}
              >
                {mod.name}
              </button>
              {pending && (
                <p className="truncate text-sm text-primary" title="FOMOD — configure">
                  FOMOD — configure
                </p>
              )}
              {hasConflict && (
                <AlertTriangle size={16} className="inline text-text-warning" />
              )}
            </div>
          </div>
        </td>
      )}
      {visible("version") && (
        <td className="max-w-0 overflow-hidden px-2 py-1.5 text-xs text-text-secondary">
          <span className="block truncate" title={version}>
            {version}
          </span>
          {mod.nexus?.updateAvailable && (
            <span
              className="ml-1 text-text-warning"
              title={`Update: ${mod.nexus.latestVersion ?? "available"}`}
            >
              ↑
            </span>
          )}
        </td>
      )}
      {visible("author") && (
        <td className="max-w-0 overflow-hidden px-2 py-1.5 text-xs text-text-secondary">
          <span className="block truncate" title={author}>
            {author}
          </span>
        </td>
      )}
      {visible("category") && (
        <td className="max-w-0 overflow-hidden px-2 py-1.5 text-xs text-text-secondary">
          <span className="block truncate" title={category}>
            {category}
          </span>
        </td>
      )}
      {visible("actions") && (
        <td className="px-2 py-1.5">
          <div className="flex items-center gap-1">
            {pending && mod.slug && onConfigureFomod && (
              <Button
                variant="secondary"
                size="sm"
                className="h-7 shrink-0 text-xs"
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
                  className="inline-flex h-7 shrink-0 items-center gap-0.5 rounded-lg bg-primary px-2 text-sm text-white hover:bg-primary-hover"
                  onClick={(e) => e.stopPropagation()}
                >
                  Remove
                  <ChevronDown size={16} />
                </button>
              </DropdownTrigger>
              <DropdownContent align="end">
                <DropdownItem
                  onClick={() => onRemove(mod.id)}
                  className="text-text-error"
                >
                  Remove from library
                </DropdownItem>
                {mod.slug && onOpenFolder && (
                  <DropdownItem onClick={() => onOpenFolder(mod)}>
                    Open folder
                  </DropdownItem>
                )}
                {onReinstall && (
                  <DropdownItem onClick={() => onReinstall(mod)}>
                    Rescan from staging
                  </DropdownItem>
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
