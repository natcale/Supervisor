/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useCallback, useState } from "react";
import { FolderDown } from "lucide-react";
import { ingestForGame, pickModPaths } from "@/lib/drop";
import { formatInvokeError } from "@/lib/errors";

type Props = {
  gameId: string;
  onIngested: (mods: import("@/types").ModManifest[], stagingDir: string) => void;
  compact?: boolean;
};

export function ModDropzone({ gameId, onIngested, compact }: Props) {
  const [error, setError] = useState<string | null>(null);

  const handleDrop = useCallback(
    async (e: React.DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setError(null);
      const paths = e.dataTransfer.files.length
        ? Array.from(e.dataTransfer.files).map((f) => {
            const path = (f as File & { path?: string }).path;
            return path ?? f.name;
          })
        : [];
      if (paths.length === 0) return;
      try {
        const result = await ingestForGame(gameId, paths);
        onIngested(result.mods, result.stagingDir);
      } catch (err) {
        setError(formatInvokeError(err));
      }
    },
    [gameId, onIngested],
  );

  const pickFiles = async () => {
    setError(null);
    try {
      const paths = await pickModPaths();
      if (paths.length === 0) return;
      const result = await ingestForGame(gameId, paths);
      onIngested(result.mods, result.stagingDir);
    } catch (err) {
      setError(formatInvokeError(err));
    }
  };

  return (
    <div>
    <div
      onDragOver={(e) => {
        e.preventDefault();
        e.stopPropagation();
      }}
      onDrop={handleDrop}
      onClick={() => void pickFiles()}
      className={`flex cursor-pointer flex-col items-center justify-center bg-panel-secondary text-text-muted transition-colors hover:border-primary hover:bg-panel-hover ${
        compact ? "py-3" : "py-8"
      }`}
    >
      <FolderDown size={compact ? 24 : 32} className="mb-2 opacity-60" />
      <span className="text-sm">Drop File(s)</span>
    </div>
    {error && <p className="mt-2 text-center text-xs text-error">{error}</p>}
    </div>
  );
}
