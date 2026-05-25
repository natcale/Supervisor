/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useMemo, useState } from "react";
import { ModTableRow } from "@/features/mod-manager/ui/mod-table-row";
import { EmptyState } from "@/features/shell/ui/empty-state";
import type { ModManifest, ModTableColumn, UserFacingIssue } from "@/types";
import { ArrowDown, ArrowUp, ArrowUpDown, X } from "lucide-react";

type SortKey = "name" | "version" | "author" | "category";
type SortDir = "asc" | "desc";

function SortIcon({
  col,
  sortKey,
  sortDir,
}: {
  col: SortKey;
  sortKey: SortKey | "manual";
  sortDir: SortDir;
}) {
  if (sortKey !== col) return <ArrowUpDown size={12} className="opacity-40" />;
  return sortDir === "asc" ? <ArrowUp size={12} /> : <ArrowDown size={12} />;
}

type Props = {
  mods: ModManifest[];
  enabledIds: string[];
  columns: ModTableColumn[];
  selectedIds: Set<string>;
  conflictModIds?: Set<string>;
  onSelect: (modId: string, additive: boolean, range: boolean) => void;
  onToggle: (modId: string) => void;
  onRemove: (modId: string) => void;
  onConfigureFomod?: (mod: ModManifest) => void;
  onShowInfo?: (mod: ModManifest) => void;
  onReinstall?: (mod: ModManifest) => void;
  onOpenFolder?: (mod: ModManifest) => void;
  onReorder?: (modIds: string[]) => void;
  globalFilter?: string;
};

export function ModTable({
  mods,
  enabledIds,
  columns,
  selectedIds,
  conflictModIds,
  onSelect,
  onToggle,
  onRemove,
  onConfigureFomod,
  onShowInfo,
  onReinstall,
  onOpenFolder,
  onReorder,
  globalFilter = "",
}: Props) {
  const [tipDismissed, setTipDismissed] = useState(false);
  const [sortKey, setSortKey] = useState<SortKey | "manual">("manual");
  const [sortDir, setSortDir] = useState<SortDir>("asc");
  const [filters, setFilters] = useState<Record<string, string>>({});
  const [dragIndex, setDragIndex] = useState<number | null>(null);

  const visible = (id: string) => columns.find((c) => c.id === id)?.visible !== false;

  const filtered = useMemo(() => {
    let list = [...mods];
    const q = globalFilter.trim().toLowerCase();
    if (q) {
      list = list.filter(
        (m) =>
          m.name.toLowerCase().includes(q) ||
          m.nexus?.author?.toLowerCase().includes(q) ||
          m.nexus?.category?.toLowerCase().includes(q),
      );
    }
    for (const [key, val] of Object.entries(filters)) {
      if (!val.trim()) continue;
      const needle = val.toLowerCase();
      list = list.filter((m) => {
        switch (key) {
          case "name":
            return m.name.toLowerCase().includes(needle);
          case "version":
            return (m.nexus?.version ?? "").toLowerCase().includes(needle);
          case "author":
            return (m.nexus?.author ?? "").toLowerCase().includes(needle);
          case "category":
            return (m.nexus?.category ?? "local").toLowerCase().includes(needle);
          default:
            return true;
        }
      });
    }
    if (sortKey !== "manual") {
      list.sort((a, b) => {
        let av = "";
        let bv = "";
        switch (sortKey) {
          case "name":
            av = a.name;
            bv = b.name;
            break;
          case "version":
            av = a.nexus?.version ?? "";
            bv = b.nexus?.version ?? "";
            break;
          case "author":
            av = a.nexus?.author ?? "";
            bv = b.nexus?.author ?? "";
            break;
          case "category":
            av = a.nexus?.category ?? "";
            bv = b.nexus?.category ?? "";
            break;
        }
        const cmp = av.localeCompare(bv, undefined, { sensitivity: "base" });
        return sortDir === "asc" ? cmp : -cmp;
      });
    }
    return list;
  }, [mods, globalFilter, filters, sortKey, sortDir]);

  const toggleSort = (key: SortKey) => {
    if (sortKey === key) setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    else {
      setSortKey(key);
      setSortDir("asc");
    }
  };

  const moveMod = (from: number, to: number) => {
    if (!onReorder || sortKey !== "manual") return;
    if (from === to || from < 0 || to < 0 || from >= filtered.length || to >= filtered.length) {
      return;
    }
    const ids = mods.map((m) => m.id);
    const draggedId = filtered[from]!.id;
    const targetId = filtered[to]!.id;
    const fromIdx = ids.indexOf(draggedId);
    const toIdx = ids.indexOf(targetId);
    if (fromIdx < 0 || toIdx < 0) return;
    ids.splice(fromIdx, 1);
    ids.splice(toIdx, 0, draggedId);
    onReorder(ids);
  };

  if (mods.length === 0) {
    return (
      <EmptyState
        slotId="view.mods.empty"
        iconSrc="/assets/icons/mods.svg"
        iconWidth={64}
        iconHeight={69}
        message="No mods in library. Drop files below or use + to add"
        link={{
          label: "View Walkthrough",
          href: "https://github.com/Nexus-Mods/Vortex/wiki/MODDINGWIKI-Users-FAQ",
        }}
        align="start"
      />
    );
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      {!tipDismissed && (
        <div className="flex shrink-0 items-center justify-between bg-content-panel px-3 py-1.5 text-sm text-text-secondary">
          <span>
            Drag the grip to set load order. Click column headers to sort instead.
          </span>
          <button type="button" onClick={() => setTipDismissed(true)} className="text-text-muted">
            <X size={14} />
          </button>
        </div>
      )}

      <div className="min-h-0 flex-1 overflow-auto">
        <table className="w-full min-w-[640px] border-collapse text-left">
          <thead className="sticky top-0 z-10 bg-content-panel">
            <tr className="text-sm text-text-muted">
              {onReorder && sortKey === "manual" && (
                <th className="w-8 px-1 py-2 font-normal" aria-label="Reorder" />
              )}
              {visible("enabled") && (
                <th className="px-2 py-2 font-normal" style={{ width: "100px" }}>
                  Enabled
                </th>
              )}
              {visible("name") && (
                <th className="px-2 py-2 font-normal">
                  <button type="button" className="inline-flex items-center gap-1" onClick={() => toggleSort("name")}>
                    Mod Name <SortIcon col="name" sortKey={sortKey} sortDir={sortDir} />
                  </button>
                </th>
              )}
              {visible("version") && (
                <th className="px-2 py-2 font-normal" style={{ width: "120px" }}>
                  <button type="button" className="inline-flex items-center gap-1" onClick={() => toggleSort("version")}>
                    Version <SortIcon col="version" sortKey={sortKey} sortDir={sortDir} />
                  </button>
                </th>
              )}
              {visible("author") && (
                <th className="px-2 py-2 font-normal" style={{ width: "140px" }}>
                  <button type="button" className="inline-flex items-center gap-1" onClick={() => toggleSort("author")}>
                    Author <SortIcon col="author" sortKey={sortKey} sortDir={sortDir} />
                  </button>
                </th>
              )}
              {visible("category") && (
                <th className="px-2 py-2 font-normal" style={{ width: "120px" }}>
                  <button
                    type="button"
                    className="inline-flex items-center gap-1"
                    onClick={() => toggleSort("category")}
                  >
                    Category <SortIcon col="category" sortKey={sortKey} sortDir={sortDir} />
                  </button>
                </th>
              )}
              {visible("actions") && (
                <th className="px-2 py-2 font-normal" style={{ width: "120px" }}>
                  Actions
                </th>
              )}
            </tr>
            <tr className="bg-content-panel">
              {onReorder && sortKey === "manual" && <th className="p-1" />}
              {visible("enabled") && <th className="p-1" />}
              {visible("name") && (
                <th className="p-1">
                  <FilterInput
                    value={filters.name ?? ""}
                    onChange={(v) => setFilters((f) => ({ ...f, name: v }))}
                  />
                </th>
              )}
              {visible("version") && (
                <th className="p-1">
                  <FilterInput
                    value={filters.version ?? ""}
                    onChange={(v) => setFilters((f) => ({ ...f, version: v }))}
                  />
                </th>
              )}
              {visible("author") && (
                <th className="p-1">
                  <FilterInput
                    value={filters.author ?? ""}
                    onChange={(v) => setFilters((f) => ({ ...f, author: v }))}
                  />
                </th>
              )}
              {visible("category") && (
                <th className="p-1">
                  <FilterInput
                    value={filters.category ?? ""}
                    onChange={(v) => setFilters((f) => ({ ...f, category: v }))}
                  />
                </th>
              )}
              {visible("actions") && <th className="p-1" />}
            </tr>
          </thead>
          <tbody>
            {filtered.map((mod, index) => (
              <ModTableRow
                key={mod.id}
                mod={mod}
                enabled={enabledIds.includes(mod.id)}
                selected={selectedIds.has(mod.id)}
                columns={columns}
                hasConflict={conflictModIds?.has(mod.id)}
                draggable={Boolean(onReorder && sortKey === "manual")}
                dragging={dragIndex === index}
                onDragStart={() => setDragIndex(index)}
                onDragOver={(e) => e.preventDefault()}
                onDrop={() => {
                  if (dragIndex !== null) moveMod(dragIndex, index);
                  setDragIndex(null);
                }}
                onDragEnd={() => setDragIndex(null)}
                onSelect={onSelect}
                onToggle={onToggle}
                onRemove={onRemove}
                onConfigureFomod={onConfigureFomod}
                onShowInfo={onShowInfo}
                onReinstall={onReinstall}
                onOpenFolder={onOpenFolder}
              />
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function FilterInput({
  value,
  onChange,
}: {
  value: string;
  onChange: (v: string) => void;
}) {
  return (
    <input
      type="text"
      value={value}
      onChange={(e) => onChange(e.target.value)}
      placeholder="Filter…"
      className="h-6 w-full bg-input-bg px-1.5 font-normal text-md text-text-primary outline-none"
    />
  );
}

export function conflictModIdsFromIssues(issues: UserFacingIssue[]): Set<string> {
  const ids = new Set<string>();
  for (const issue of issues) {
    if (issue.id.startsWith("conflict-")) {
      for (const choice of issue.choices) {
        if (choice.id.startsWith("prefer-")) {
          ids.add(choice.id.replace(/^prefer-/, ""));
        }
      }
    }
  }
  return ids;
}
