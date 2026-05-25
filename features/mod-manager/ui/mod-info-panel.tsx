/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { enrichModMetadata } from "@/lib/api/nexus";
import { setModNotes } from "@/lib/api/mods";
import type { ModManifest } from "@/types";
import { open } from "@tauri-apps/plugin-shell";
import { ExternalLink, RefreshCw, X } from "lucide-react";

type Props = {
  mod: ModManifest | null;
  gameId: string;
  gameDomain?: string;
  onClose: () => void;
  onOpenFolder?: (mod: ModManifest) => void;
  onModUpdated?: (mod: ModManifest) => void;
};

export function ModInfoPanel({
  mod,
  gameId,
  gameDomain,
  onClose,
  onOpenFolder,
  onModUpdated,
}: Props) {
  const [notes, setNotes] = useState("");
  const [savingNotes, setSavingNotes] = useState(false);
  const [enriching, setEnriching] = useState(false);

  useEffect(() => {
    setNotes(mod?.notes ?? "");
  }, [mod?.id, mod?.notes]);

  if (!mod) return null;

  const domain = mod.nexus?.domain ?? gameDomain;

  const saveNotes = async () => {
    setSavingNotes(true);
    try {
      const library = await setModNotes(gameId, mod.id, notes.trim() || null);
      const updated = library.mods.find((m) => m.id === mod.id);
      if (updated) {
        onModUpdated?.({
          ...mod,
          notes: updated.notes,
        });
      }
    } catch (e) {
      console.error(e);
    } finally {
      setSavingNotes(false);
    }
  };

  const fetchFromNexus = async () => {
    if (!mod.nexus) return;
    setEnriching(true);
    try {
      const meta = await enrichModMetadata(gameId, mod.id);
      onModUpdated?.({
        ...mod,
        nexus: { ...mod.nexus, ...meta },
      });
    } catch (e) {
      console.error(e);
    } finally {
      setEnriching(false);
    }
  };

  return (
    <div className="shrink-0 border-b border-border bg-panel-secondary p-3">
      <div className="mb-2 flex items-start justify-between gap-2">
        <div className="min-w-0 flex-1">
          <h3 className="truncate text-sm font-medium text-text-primary">{mod.name}</h3>
          {mod.nexus?.updateAvailable && (
            <p className="mt-0.5 text-xs text-[var(--warning)]">
              Update available
              {mod.nexus.latestVersion ? `: ${mod.nexus.latestVersion}` : ""}
            </p>
          )}
        </div>
        <button
          type="button"
          onClick={onClose}
          className="text-text-muted hover:text-text-primary"
        >
          <X size={16} />
        </button>
      </div>

      {mod.nexus?.summary && (
        <p className="mb-2 text-xs text-text-secondary">{mod.nexus.summary}</p>
      )}

      <dl className="mb-3 grid grid-cols-2 gap-x-4 gap-y-1 text-xs">
        <dt className="text-text-muted">Files</dt>
        <dd className="text-text-primary">{mod.files.length}</dd>
        <dt className="text-text-muted">Version</dt>
        <dd className="text-text-primary">{mod.nexus?.version ?? "—"}</dd>
        <dt className="text-text-muted">Author</dt>
        <dd className="text-text-primary">{mod.nexus?.author ?? "—"}</dd>
        <dt className="text-text-muted">Category</dt>
        <dd className="text-text-primary">{mod.nexus?.category ?? "Local"}</dd>
        <dt className="text-text-muted">Install path</dt>
        <dd className="truncate font-mono text-text-primary">{mod.slug ?? mod.id}</dd>
      </dl>

      <div className="mb-3">
        <label className="mb-1 block text-xs text-text-muted">Notes</label>
        <textarea
          value={notes}
          onChange={(e) => setNotes(e.target.value)}
          onBlur={() => void saveNotes()}
          placeholder="Add personal notes about this mod…"
          rows={2}
          className="w-full resize-none rounded-lg bg-input-bg px-2 py-1.5 text-xs text-text-primary outline-none focus:ring-1 focus:ring-[var(--primary)]"
        />
        {savingNotes && (
          <p className="mt-0.5 text-xs text-text-muted">Saving…</p>
        )}
      </div>

      <div className="flex flex-wrap gap-2">
        {mod.slug && onOpenFolder && (
          <Button variant="secondary" size="sm" className="h-7 text-xs" onClick={() => onOpenFolder(mod)}>
            Open folder
          </Button>
        )}
        {mod.nexus && (
          <Button
            variant="ghost"
            size="sm"
            className="h-7 text-xs"
            disabled={enriching}
            onClick={() => void fetchFromNexus()}
          >
            <RefreshCw size={14} className={`mr-1 ${enriching ? "animate-spin" : ""}`} />
            Fetch from Nexus
          </Button>
        )}
        {domain && mod.nexus && (
          <Button
            variant="ghost"
            size="sm"
            className="h-7 text-xs"
            onClick={() =>
              void open(`https://www.nexusmods.com/${domain}/mods/${mod.nexus!.modId}`)
            }
          >
            <ExternalLink size={14} className="mr-1" />
            View on Nexus
          </Button>
        )}
      </div>
    </div>
  );
}
