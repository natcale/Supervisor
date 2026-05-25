/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useMemo, useState } from "react";
import { Button } from "@/components/ui/button";
import type { DetectedGame } from "@/types";
import { steamCoverUrl, withCover } from "@/types";
import { ArrowDown, ArrowUp, ArrowUpDown, FolderOpen, RefreshCw, Trash2 } from "lucide-react";

type SortKey = "name" | "platform" | "installPath";
type SortDir = "asc" | "desc";

type Props = {
  games: DetectedGame[];
  selectedId?: string;
  refreshing?: boolean;
  /** Cap scroll area height; list stays short when fewer games are shown. */
  maxListHeight?: string;
  onSelect: (game: DetectedGame) => void;
  onRefresh: () => void;
  onAddLocal: () => void;
  onRemove: (game: DetectedGame) => void;
  onOpenFolder: (game: DetectedGame) => void;
  onUpdateNexusDomain?: (game: DetectedGame, domain: string) => void | Promise<void>;
};

function SortIcon({
  col,
  sortKey,
  sortDir,
}: {
  col: SortKey;
  sortKey: SortKey;
  sortDir: SortDir;
}) {
  if (sortKey !== col) return <ArrowUpDown size={12} className="opacity-40" />;
  return sortDir === "asc" ? <ArrowUp size={12} /> : <ArrowDown size={12} />;
}

function platformLabel(platform: DetectedGame["platform"]) {
  switch (platform) {
    case "steam":
      return "Steam";
    case "epic":
      return "Epic";
    case "gog":
      return "GOG";
    case "heroic":
      return "Heroic";
    case "manual":
      return "Local";
    default:
      return platform;
  }
}

function GameCover({ cover, name }: { cover?: string | null; name: string }) {
  const [broken, setBroken] = useState(false);
  const showImage = Boolean(cover) && !broken;

  return showImage ? (
    <img
      src={cover!}
      alt=""
      className="aspect-[460/215] w-[184px] max-w-full rounded object-cover"
      loading="lazy"
      onError={() => setBroken(true)}
    />
  ) : (
    <div className="flex aspect-[460/215] w-[184px] max-w-full items-center justify-center rounded bg-panel-secondary text-xs font-medium text-text-muted">
      {name.slice(0, 2).toUpperCase()}
    </div>
  );
}

export function GamesTable({
  games,
  selectedId,
  refreshing = false,
  maxListHeight,
  onSelect,
  onRefresh,
  onAddLocal,
  onRemove,
  onOpenFolder,
  onUpdateNexusDomain,
}: Props) {
  const [sortKey, setSortKey] = useState<SortKey>("name");
  const [sortDir, setSortDir] = useState<SortDir>("asc");
  const [filter, setFilter] = useState("");

  const sorted = useMemo(() => {
    let list = [...games];
    const q = filter.trim().toLowerCase();
    if (q) {
      list = list.filter(
        (g) =>
          g.name.toLowerCase().includes(q) ||
          g.installPath.toLowerCase().includes(q) ||
          platformLabel(g.platform).toLowerCase().includes(q),
      );
    }
    list.sort((a, b) => {
      let av = "";
      let bv = "";
      switch (sortKey) {
        case "name":
          av = a.name;
          bv = b.name;
          break;
        case "platform":
          av = platformLabel(a.platform);
          bv = platformLabel(b.platform);
          break;
        case "installPath":
          av = a.installPath;
          bv = b.installPath;
          break;
      }
      const cmp = av.localeCompare(bv, undefined, { sensitivity: "base" });
      return sortDir === "asc" ? cmp : -cmp;
    });
    return list;
  }, [games, filter, sortKey, sortDir]);

  const toggleSort = (key: SortKey) => {
    if (sortKey === key) setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    else {
      setSortKey(key);
      setSortDir("asc");
    }
  };

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      <div className="flex shrink-0 items-center gap-2 px-2 py-2">
        <Button variant="secondary" size="sm" disabled={refreshing} onClick={onRefresh}>
          <RefreshCw size={16} className={`${refreshing ? "animate-spin" : ""}`} />
          Refresh scan
        </Button>
        <Button variant="secondary" size="sm" onClick={onAddLocal}>
          Add local game
        </Button>
        <input
          type="search"
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          placeholder="Filter games…"
          className="ml-auto w-48 rounded-lg bg-input-bg px-2 py-1 text-sm text-text-primary outline-none focus:border-primary"
        />
      </div>

      {games.length === 0 ? (
        <div className="flex flex-1 items-center justify-center py-12 text-sm text-text-muted">
          No games found. Refresh scan or add a local game folder.
        </div>
      ) : (
        <div
          className="overflow-auto"
          style={maxListHeight ? { maxHeight: maxListHeight } : undefined}
        >
          <table className="w-full min-w-[720px] border-collapse text-left">
            <thead className="sticky top-0 z-10 bg-panel-secondary">
              <tr className="text-sm text-text-muted">
                <th className="w-[188px] px-2 py-2 font-normal">Cover</th>
                <th className="px-2 py-2 font-normal">
                  <button type="button" className="inline-flex items-center gap-1" onClick={() => toggleSort("name")}>
                    Game <SortIcon col="name" sortKey={sortKey} sortDir={sortDir} />
                  </button>
                </th>
                <th className="px-2 py-2 font-normal" style={{ width: "100px" }}>
                  <button type="button" className="inline-flex items-center gap-1" onClick={() => toggleSort("platform")}>
                    Source <SortIcon col="platform" sortKey={sortKey} sortDir={sortDir} />
                  </button>
                </th>
                <th className="px-2 py-2 font-normal">
                  <button type="button" className="inline-flex items-center gap-1" onClick={() => toggleSort("installPath")}>
                    Install path <SortIcon col="installPath" sortKey={sortKey} sortDir={sortDir} />
                  </button>
                </th>
                <th className="px-2 py-2 font-normal" style={{ width: "160px" }}>
                  Nexus domain
                </th>
                <th className="w-40 px-2 py-2 font-normal">Actions</th>
              </tr>
            </thead>
            <tbody>
              {sorted.map((game) => {
                const enriched = withCover(game);
                const cover = enriched.coverUrl ?? steamCoverUrl(game.appId);
                const selected = selectedId === game.id;
                return (
                  <tr
                    key={game.id}
                    className={`text-sm ${selected ? "bg-panel-hover/60" : "hover:bg-panel-hover/40"}`}
                  >
                    <td className="px-2 py-2 align-middle">
                      <GameCover cover={cover} name={game.name} />
                    </td>
                    <td className="px-2 py-2">
                      <button
                        type="button"
                        className="text-left text-text-primary hover:underline"
                        onClick={() => onSelect(game)}
                      >
                        {game.name}
                      </button>
                    </td>
                    <td className="px-2 py-2 text-text-secondary">{platformLabel(game.platform)}</td>
                    <td className="max-w-xs truncate px-2 py-2 text-text-muted" title={game.installPath}>
                      {game.installPath}
                    </td>
                    <td className="px-2 py-2">
                      {game.platform === "manual" && onUpdateNexusDomain ? (
                        <input
                          type="text"
                          defaultValue={game.nexusDomain ?? ""}
                          placeholder="skyrimspecialedition"
                          className="w-full rounded border border-border bg-panel-secondary px-2 py-1 text-xs text-text-primary outline-none focus:border-primary"
                          onBlur={(e) => {
                            const next = e.target.value.trim();
                            const current = game.nexusDomain ?? "";
                            if (next !== current) {
                              void onUpdateNexusDomain(game, next);
                            }
                          }}
                          onKeyDown={(e) => {
                            if (e.key === "Enter") {
                              e.currentTarget.blur();
                            }
                          }}
                        />
                      ) : (
                        <span className="text-xs text-text-muted">{game.nexusDomain ?? "—"}</span>
                      )}
                    </td>
                    <td className="px-2 py-2">
                      <div className="flex items-center gap-1">
                        <button
                          type="button"
                          title="Open install folder"
                          className="rounded p-1 text-text-muted hover:bg-panel-hover hover:text-text-primary"
                          onClick={() => onOpenFolder(game)}
                        >
                          <FolderOpen size={16} />
                        </button>
                        {game.platform === "manual" && (
                          <button
                            type="button"
                            title="Remove local game"
                            className="rounded p-1 text-text-muted hover:bg-panel-hover hover:text-error"
                            onClick={() => onRemove(game)}
                          >
                            <Trash2 size={16} />
                          </button>
                        )}
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
