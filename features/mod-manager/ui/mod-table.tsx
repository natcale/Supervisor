/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { ModTableRow } from "@/features/mod-manager/ui/mod-table-row";
import { EmptyState } from "@/features/shell/ui/empty-state";
import type { ModFilter } from "@/features/mod-manager/ui/mod-filter-bar";
import type { ModManifest, ModTableColumn, UserFacingIssue } from "@/types";
import { ArrowDown, ArrowUp, ArrowUpDown, RotateCcw, X } from "lucide-react";

const WALKTHROUGH_URL =
  "https://github.com/natcale/Supervisor/tree/main/docs/user";

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
  if (sortKey !== col) return <ArrowUpDown size={16} />;
  return sortDir === "asc" ? <ArrowUp size={16} /> : <ArrowDown size={16} />;
}

type Props = {
  mods: ModManifest[];
  libraryModCount: number;
  viewFilter?: ModFilter;
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
  libraryMods?: ModManifest[];
  globalFilter?: string;
  onClearGlobalFilter?: () => void;
  onClearViewFilter?: () => void;
};

export function ModTable({
  mods,
  libraryModCount,
  viewFilter = "all",
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
  libraryMods,
  globalFilter = "",
  onClearGlobalFilter,
  onClearViewFilter,
}: Props) {
  const [tipDismissed, setTipDismissed] = useState(false);
  const [sortKey, setSortKey] = useState<SortKey | "manual">("manual");
  const [sortDir, setSortDir] = useState<SortDir>("asc");
  const [filters, setFilters] = useState<Record<string, string>>({});
  const [dragIndex, setDragIndex] = useState<number | null>(null);
  const [dropIndex, setDropIndex] = useState<number | null>(null);
  const dragSourceRef = useRef<number | null>(null);
  const dropTargetRef = useRef<number | null>(null);

  const orderSource = libraryMods ?? mods;

  const visible = (id: string) =>
    columns.find((c) => c.id === id)?.visible !== false;

  const hasColumnFilters = Object.values(filters).some((v) => v.trim());
  const hasGlobalFilter = globalFilter.trim().length > 0;
  const hasViewFilter = viewFilter !== "all";
  const canReorder =
    Boolean(onReorder) &&
    sortKey === "manual" &&
    !hasColumnFilters &&
    !hasGlobalFilter &&
    !hasViewFilter;

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
            return (m.nexus?.category ?? "local")
              .toLowerCase()
              .includes(needle);
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

  const resetTableView = () => {
    setSortKey("manual");
    setSortDir("asc");
    setFilters({});
    onClearGlobalFilter?.();
    onClearViewFilter?.();
  };

  const moveMod = useCallback(
    (from: number, to: number) => {
      if (!onReorder || !canReorder) return;
      if (
        from === to ||
        from < 0 ||
        to < 0 ||
        from >= filtered.length ||
        to >= filtered.length
      ) {
        return;
      }
      const ids = orderSource.map((m) => m.id);
      const draggedId = filtered[from]!.id;
      const targetId = filtered[to]!.id;
      const fromIdx = ids.indexOf(draggedId);
      let toIdx = ids.indexOf(targetId);
      if (fromIdx < 0 || toIdx < 0) return;
      ids.splice(fromIdx, 1);
      if (fromIdx < toIdx) toIdx -= 1;
      ids.splice(toIdx, 0, draggedId);
      onReorder(ids);
    },
    [canReorder, filtered, onReorder, orderSource],
  );

  const beginPointerReorder = useCallback((index: number) => {
    dragSourceRef.current = index;
    dropTargetRef.current = index;
    setDragIndex(index);
    setDropIndex(index);
  }, []);

  const updatePointerDropTarget = useCallback((index: number) => {
    if (dragSourceRef.current === null) return;
    dropTargetRef.current = index;
    setDropIndex(index);
  }, []);

  const commitPointerReorder = useCallback(() => {
    const from = dragSourceRef.current;
    const to = dropTargetRef.current;
    dragSourceRef.current = null;
    dropTargetRef.current = null;
    setDragIndex(null);
    setDropIndex(null);
    if (from !== null && to !== null) moveMod(from, to);
  }, [moveMod]);

  useEffect(() => {
    if (dragIndex === null) return;

    const onPointerMove = (e: PointerEvent) => {
      const row = document
        .elementFromPoint(e.clientX, e.clientY)
        ?.closest<HTMLElement>("[data-mod-row-index]");
      if (!row) return;
      const index = Number(row.dataset.modRowIndex);
      if (!Number.isNaN(index)) updatePointerDropTarget(index);
    };

    const onPointerUp = () => commitPointerReorder();

    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", onPointerUp);
    window.addEventListener("pointercancel", onPointerUp);
    return () => {
      window.removeEventListener("pointermove", onPointerMove);
      window.removeEventListener("pointerup", onPointerUp);
      window.removeEventListener("pointercancel", onPointerUp);
    };
  }, [dragIndex, commitPointerReorder, updatePointerDropTarget]);

  if (libraryModCount === 0) {
    return (
      <EmptyState
        slotId="view.mods.empty"
        iconSrc="/assets/icons/mods.svg"
        iconWidth={64}
        iconHeight={69}
        message="No mods in library. Drop files below or use + to add"
        link={{
          label: "View Walkthrough",
          href: WALKTHROUGH_URL,
        }}
        align="start"
      />
    );
  }

  if (mods.length === 0) {
    return (
      <EmptyState
        iconSrc="/assets/icons/mods.svg"
        iconWidth={64}
        iconHeight={69}
        message={filteredEmptyMessage(viewFilter)}
        align="start"
      />
    );
  }

  if (filtered.length === 0) {
    return (
      <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
        {!canReorder && (
          <LoadOrderAdvisory
            onReset={resetTableView}
            onDismiss={() => setTipDismissed(true)}
            dismissed={tipDismissed}
          />
        )}
        <div className="flex flex-1 items-center justify-center p-8">
          <p className="text-base text-text-secondary">
            No mods match the current filters. Clear search or column filters to
            see more.
          </p>
        </div>
      </div>
    );
  }

  const showLoadOrderTip = canReorder && !tipDismissed;
  const showLoadOrderWarning = !canReorder && !tipDismissed;

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      {showLoadOrderTip && (
        <div className="flex shrink-0 items-center justify-between bg-content-panel px-3 py-1.5 text-sm text-text-secondary">
          <span>
            Drag the grip to set load order. Click column headers to sort
            instead.
          </span>
          <button
            type="button"
            onClick={() => setTipDismissed(true)}
            className="text-text-muted"
            aria-label="Dismiss tip"
          >
            <X size={14} />
          </button>
        </div>
      )}
      {showLoadOrderWarning && (
        <LoadOrderAdvisory
          onReset={resetTableView}
          onDismiss={() => setTipDismissed(true)}
          dismissed={tipDismissed}
        />
      )}

      <div className="min-h-0 flex-1 overflow-auto">
        <table className="w-full min-w-[640px] table-fixed border-collapse text-left">
          <colgroup>
            {canReorder && <col className="w-8" />}
            {visible("enabled") && <col className="w-[100px]" />}
            {visible("name") && <col />}
            {visible("version") && <col className="w-[120px]" />}
            {visible("author") && <col className="w-[140px]" />}
            {visible("category") && <col className="w-[120px]" />}
            {visible("actions") && <col className="w-[120px]" />}
            {!canReorder && onReorder && <col className="w-10" />}
          </colgroup>
          <thead className="sticky top-0 z-10 bg-content-panel">
            <tr className="text-sm text-text-muted">
              {canReorder && (
                <th
                  className="w-8 px-1 py-2 font-normal"
                  aria-label="Reorder"
                />
              )}
              {visible("enabled") && (
                <th
                  className="px-2 py-2 font-normal"
                  style={{ width: "100px" }}
                >
                  Enabled
                </th>
              )}
              {visible("name") && (
                <th className="px-2 py-2 font-normal">
                  <button
                    type="button"
                    className="inline-flex items-center gap-1"
                    onClick={() => toggleSort("name")}
                  >
                    Mod Name{" "}
                    <SortIcon col="name" sortKey={sortKey} sortDir={sortDir} />
                  </button>
                </th>
              )}
              {visible("version") && (
                <th
                  className="px-2 py-2 font-normal"
                  style={{ width: "120px" }}
                >
                  <button
                    type="button"
                    className="inline-flex items-center gap-1"
                    onClick={() => toggleSort("version")}
                  >
                    Version{" "}
                    <SortIcon
                      col="version"
                      sortKey={sortKey}
                      sortDir={sortDir}
                    />
                  </button>
                </th>
              )}
              {visible("author") && (
                <th
                  className="px-2 py-2 font-normal"
                  style={{ width: "140px" }}
                >
                  <button
                    type="button"
                    className="inline-flex items-center gap-1"
                    onClick={() => toggleSort("author")}
                  >
                    Author{" "}
                    <SortIcon
                      col="author"
                      sortKey={sortKey}
                      sortDir={sortDir}
                    />
                  </button>
                </th>
              )}
              {visible("category") && (
                <th
                  className="px-2 py-2 font-normal"
                  style={{ width: "120px" }}
                >
                  <button
                    type="button"
                    className="inline-flex items-center gap-1"
                    onClick={() => toggleSort("category")}
                  >
                    Category{" "}
                    <SortIcon
                      col="category"
                      sortKey={sortKey}
                      sortDir={sortDir}
                    />
                  </button>
                </th>
              )}
              {visible("actions") && (
                <th
                  className="px-2 py-2 font-normal"
                  style={{ width: "120px" }}
                >
                  Actions
                </th>
              )}
              {!canReorder && onReorder && (
                <th className="w-10 px-2 py-2 font-normal">
                  <button
                    type="button"
                    title="Reset to load order view"
                    onClick={resetTableView}
                    className="inline-flex items-center justify-center rounded-lg p-1 text-text-muted hover:bg-panel-hover hover:text-text-primary"
                  >
                    <RotateCcw size={14} />
                  </button>
                </th>
              )}
            </tr>
            <tr className="bg-content-panel">
              {canReorder && <th className="p-1" />}
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
              {!canReorder && onReorder && <th className="p-1" />}
            </tr>
          </thead>
          <tbody>
            {filtered.map((mod, index) => (
              <ModTableRow
                key={mod.id}
                rowIndex={index}
                mod={mod}
                enabled={enabledIds.includes(mod.id)}
                selected={selectedIds.has(mod.id)}
                columns={columns}
                hasConflict={conflictModIds?.has(mod.id)}
                reorderable={canReorder}
                dragging={dragIndex === index}
                dropTarget={dropIndex === index && dragIndex !== index}
                onGripPointerDown={beginPointerReorder}
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

function LoadOrderAdvisory({
  onReset,
  onDismiss,
  dismissed,
}: {
  onReset: () => void;
  onDismiss: () => void;
  dismissed: boolean;
}) {
  if (dismissed) return null;

  return (
    <div className="flex shrink-0 items-center justify-between border-b border-border px-2 py-1.5 text-sm text-warning">
      <span>
        Load order editing is disabled while sorting or filters are active.
        Reset to drag mods into load order.
      </span>
      <div className="flex shrink-0 items-center gap-2">
        <button
          type="button"
          onClick={onDismiss}
          className="text-text-muted"
          aria-label="Dismiss"
        >
          <X size={16} />
        </button>
      </div>
    </div>
  );
}

function filteredEmptyMessage(filter: ModFilter): string {
  switch (filter) {
    case "enabled":
      return "No enabled mods in this view.";
    case "disabled":
      return "No disabled mods in this view.";
    case "conflicts":
      return "No mods with conflicts in this view.";
    case "fomod":
      return "No mods pending FOMOD configuration.";
    case "updates":
      return "No mods with available updates.";
    default:
      return "No mods match the current view.";
  }
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
      className="h-6 w-full rounded-lg bg-input-bg px-1.5 font-normal text-md text-text-primary outline-none"
    />
  );
}

export function conflictModIdsFromIssues(
  issues: UserFacingIssue[],
): Set<string> {
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
