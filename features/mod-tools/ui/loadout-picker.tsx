/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useCallback, useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Dropdown,
  DropdownContent,
  DropdownItem,
  DropdownSeparator,
  DropdownTrigger,
} from "@/components/ui/dropdown";
import { createLoadout, deleteLoadout, listLoadouts, switchLoadout } from "@/lib/api/loadouts";
import type { Loadout, LoadoutSummary } from "@/types";
import { Layers } from "lucide-react";

type Props = {
  gameId: string;
  activeLoadoutId?: string;
  loadoutName?: string;
  onLoadoutChange: (loadout: Loadout) => void;
};

export function LoadoutPicker({
  gameId,
  activeLoadoutId,
  loadoutName,
  onLoadoutChange,
}: Props) {
  const [summaries, setSummaries] = useState<LoadoutSummary[]>([]);
  const [activeId, setActiveId] = useState(activeLoadoutId ?? "default");
  const [creating, setCreating] = useState(false);
  const [newName, setNewName] = useState("");

  const refresh = useCallback(async () => {
    try {
      const [list, current] = await listLoadouts(gameId);
      setSummaries(list);
      setActiveId(current);
    } catch (e) {
      console.error(e);
    }
  }, [gameId]);

  useEffect(() => {
    void refresh();
  }, [refresh, activeLoadoutId]);

  const switchTo = async (id: string) => {
    const loadout = await switchLoadout(gameId, id);
    setActiveId(loadout.id);
    onLoadoutChange(loadout);
    void refresh();
  };

  const createNew = async () => {
    const name = newName.trim();
    if (!name) return;
    const loadout = await createLoadout(gameId, name);
    setCreating(false);
    setNewName("");
    await switchTo(loadout.id);
  };

  const removeLoadout = async (id: string) => {
    if (id === "default") return;
    await deleteLoadout(gameId, id);
    if (activeId === id) {
      await switchTo("default");
    } else {
      void refresh();
    }
  };

  const label = loadoutName ?? summaries.find((s) => s.id === activeId)?.name ?? "Default";

  return (
    <Dropdown>
      <DropdownTrigger asChild>
        <Button variant="ghost" size="sm">
          <Layers size={14} className="mr-1" />
          {label}
        </Button>
      </DropdownTrigger>
      <DropdownContent align="end" className="min-w-[200px]">
        {summaries.map((s) => (
          <DropdownItem key={s.id} onClick={() => void switchTo(s.id)}>
            <span className="flex min-w-0 flex-1 items-center gap-2">
              <span className="truncate">{s.name}</span>
              {s.id === activeId && <span className="text-primary">✓</span>}
              <span className="ml-auto text-xs text-text-muted">{s.enabledCount}</span>
            </span>
          </DropdownItem>
        ))}
        <DropdownSeparator />
        {creating ? (
          <div className="flex gap-1 p-2" onClick={(e) => e.stopPropagation()}>
            <input
              autoFocus
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") void createNew();
                if (e.key === "Escape") setCreating(false);
              }}
              placeholder="Loadout name"
              className="min-w-0 flex-1 rounded bg-input-bg px-2 py-1 text-xs outline-none"
            />
            <Button variant="accent" size="sm" className="h-7 px-2 text-xs" onClick={() => void createNew()}>
              Add
            </Button>
          </div>
        ) : (
          <DropdownItem onClick={() => setCreating(true)}>New loadout…</DropdownItem>
        )}
        {summaries
          .filter((s) => s.id !== "default")
          .map((s) => (
            <DropdownItem
              key={`del-${s.id}`}
              className="text-[var(--error)]"
              onClick={() => void removeLoadout(s.id)}
            >
              Delete {s.name}
            </DropdownItem>
          ))}
      </DropdownContent>
    </Dropdown>
  );
}
