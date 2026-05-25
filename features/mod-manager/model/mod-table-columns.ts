/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import type { ModTableColumn } from "@/types";
import { DEFAULT_MOD_COLUMNS } from "@/types";

const STORAGE_KEY = "supervisor-mod-columns";

export function loadModColumns(): ModTableColumn[] {
  if (typeof window === "undefined") return DEFAULT_MOD_COLUMNS;
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return DEFAULT_MOD_COLUMNS;
    const parsed = JSON.parse(raw) as ModTableColumn[];
    const ids = new Set(parsed.map((c) => c.id));
    const merged = DEFAULT_MOD_COLUMNS.map((def) => {
      const saved = parsed.find((c) => c.id === def.id);
      return saved ? { ...def, visible: saved.visible } : def;
    });
    for (const col of parsed) {
      if (!ids.has(col.id) && DEFAULT_MOD_COLUMNS.every((d) => d.id !== col.id)) {
        merged.push(col);
      }
    }
    return merged;
  } catch {
    return DEFAULT_MOD_COLUMNS;
  }
}

export function saveModColumns(columns: ModTableColumn[]) {
  if (typeof window === "undefined") return;
  localStorage.setItem(STORAGE_KEY, JSON.stringify(columns.map((c) => ({ id: c.id, visible: c.visible }))));
}

export function toggleColumn(columns: ModTableColumn[], id: string): ModTableColumn[] {
  return columns.map((c) => (c.id === id ? { ...c, visible: !c.visible } : c));
}
