/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

export type ModFilter = "all" | "enabled" | "disabled" | "conflicts" | "fomod" | "updates";

const FILTERS: { id: ModFilter; label: string }[] = [
  { id: "all", label: "All" },
  { id: "enabled", label: "Enabled" },
  { id: "disabled", label: "Disabled" },
  { id: "conflicts", label: "Conflicts" },
  { id: "fomod", label: "FOMOD" },
  { id: "updates", label: "Updates" },
];

type Props = {
  active: ModFilter;
  counts: Partial<Record<ModFilter, number>>;
  onChange: (filter: ModFilter) => void;
};

export function ModFilterBar({ active, counts, onChange }: Props) {
  return (
    <div className="flex shrink-0 flex-wrap gap-1 border-b border-border px-2 py-1.5">
      {FILTERS.map(({ id, label }) => {
        const count = counts[id];
        const isActive = active === id;
        return (
          <button
            key={id}
            type="button"
            onClick={() => onChange(id)}
            className={`rounded-full px-2.5 py-0.5 text-xs transition-colors ${
              isActive
                ? "bg-primary text-white"
                : "text-text-muted hover:bg-panel-hover hover:text-text-secondary"
            }`}
          >
            {label}
            {count !== undefined && count > 0 && (
              <span className={`ml-1 ${isActive ? "opacity-90" : "opacity-70"}`}>({count})</span>
            )}
          </button>
        );
      })}
    </div>
  );
}

export function applyModFilter<T extends { id: string; installState?: string; needsFomod?: boolean; nexus?: { updateAvailable?: boolean } }>(
  mods: T[],
  filter: ModFilter,
  enabledIds: string[],
  conflictModIds: Set<string>,
): T[] {
  switch (filter) {
    case "enabled":
      return mods.filter((m) => enabledIds.includes(m.id));
    case "disabled":
      return mods.filter((m) => !enabledIds.includes(m.id));
    case "conflicts":
      return mods.filter((m) => conflictModIds.has(m.id));
    case "fomod":
      return mods.filter((m) => m.installState === "pendingFomod" || m.needsFomod);
    case "updates":
      return mods.filter((m) => m.nexus?.updateAvailable);
    default:
      return mods;
  }
}
