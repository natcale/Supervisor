/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useCallback, useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import {
  getPluginList,
  setPluginOrder,
  sortPluginsLoot,
  togglePlugin,
} from "@/lib/api/plugins";
import { getAppSettings } from "@/lib/api/settings";
import { mergeSettings } from "@/lib/settings-defaults";
import { formatInvokeError } from "@/lib/errors";
import type { DetectedGame, ModManifest, PluginEntry } from "@/types";
import { GripVertical } from "lucide-react";

type Props = {
  game: DetectedGame;
  mods: ModManifest[];
  enabledIds: string[];
};

export function PluginList({ game, mods, enabledIds }: Props) {
  const [plugins, setPlugins] = useState<PluginEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [dragIndex, setDragIndex] = useState<number | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      let list = await getPluginList(game, mods, enabledIds);
      const gs = mergeSettings(await getAppSettings());
      if (gs.autoSortPlugins) {
        list = await sortPluginsLoot(game, mods, enabledIds);
      }
      setPlugins(list);
    } catch (e) {
      setError(formatInvokeError(e));
    } finally {
      setLoading(false);
    }
  }, [game, mods, enabledIds]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const autoSort = async () => {
    setLoading(true);
    try {
      const sorted = await sortPluginsLoot(game, mods, enabledIds);
      setPlugins(sorted);
    } finally {
      setLoading(false);
    }
  };

  const persistOrder = async (next: PluginEntry[]) => {
    setPlugins(next);
    try {
      const saved = await setPluginOrder(
        game,
        mods,
        enabledIds,
        next.map((p) => p.name),
      );
      setPlugins(saved);
    } catch (e) {
      setError(formatInvokeError(e));
      void refresh();
    }
  };

  const toggleOne = async (pluginName: string, enabled: boolean) => {
    setLoading(true);
    try {
      const nextList = await togglePlugin(game, mods, enabledIds, pluginName, enabled);
      setPlugins(nextList);
    } catch (e) {
      setError(formatInvokeError(e));
    } finally {
      setLoading(false);
    }
  };

  const movePlugin = (from: number, to: number) => {
    if (from === to || from < 0 || to < 0 || from >= plugins.length || to >= plugins.length) {
      return;
    }
    const next = [...plugins];
    const [item] = next.splice(from, 1);
    next.splice(to, 0, item!);
    void persistOrder(next);
  };

  if (plugins.length === 0 && !loading) {
    return (
      <p className="py-8 text-center text-sm text-text-muted">
        No Bethesda plugins (.esp / .esm / .esl) in enabled mods.
      </p>
    );
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="mb-3 flex items-center gap-2">
        <Button variant="secondary" size="sm" disabled={loading} onClick={() => void autoSort()}>
          {loading ? "Sorting…" : "Auto-sort (LOOT)"}
        </Button>
        <Button variant="ghost" size="sm" disabled={loading} onClick={() => void refresh()}>
          Refresh list
        </Button>
        <span className="text-xs text-text-muted">
          {plugins.length} plugin(s) · drag to reorder
        </span>
        {error && <span className="text-xs text-error">{error}</span>}
      </div>
      <ul className="min-h-0 flex-1 space-y-1 overflow-y-auto">
        {plugins.map((p, i) => (
          <li
            key={p.name}
            draggable
            onDragStart={() => setDragIndex(i)}
            onDragOver={(e) => e.preventDefault()}
            onDrop={() => {
              if (dragIndex !== null) movePlugin(dragIndex, i);
              setDragIndex(null);
            }}
            onDragEnd={() => setDragIndex(null)}
            className={`flex cursor-grab items-center gap-2 rounded-md px-2 py-2 text-sm hover:bg-panel-hover active:cursor-grabbing ${
              dragIndex === i ? "opacity-50" : ""
            }`}
          >
            <GripVertical size={14} className="shrink-0 text-text-muted" />
            <label className="flex cursor-pointer items-center gap-2">
              <input
                type="checkbox"
                checked={p.enabled}
                aria-label={`Enable plugin ${p.name}`}
                className="rounded accent-primary"
                onChange={(e) => void toggleOne(p.name, e.target.checked)}
              />
            </label>
            <span className="w-6 text-xs text-text-muted">{i + 1}</span>
            <span className="min-w-0 flex-1 truncate text-text-primary">{p.name}</span>
            {p.isMaster && <span className="text-xs text-text-muted">master</span>}
          </li>
        ))}
      </ul>
    </div>
  );
}
