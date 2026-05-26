/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { Button } from "@/components/ui/button";

type Props = {
  count: number;
  onEnableAll: () => void;
  onDisableAll: () => void;
  onRemoveAll: () => void;
  onClear: () => void;
};

export function ModBatchBar({
  count,
  onEnableAll,
  onDisableAll,
  onRemoveAll,
  onClear,
}: Props) {
  if (count === 0) return null;

  return (
    <div className="flex shrink-0 items-center gap-2 bg-accent px-5 py-2 text-sm">
      <span className="text-text-primary">{count} selected</span>
      <Button
        variant="ghost"
        size="sm"
        className="h-7 text-sm border border-white"
        onClick={onEnableAll}
      >
        Enable
      </Button>
      <Button
        variant="ghost"
        size="sm"
        className="h-7 text-sm  border-white"
        onClick={onDisableAll}
      >
        Disable
      </Button>
      <Button
        variant="ghost"
        size="sm"
        className="h-7 text-sm border border-white"
        onClick={onRemoveAll}
      >
        Remove
      </Button>
      <Button
        variant="ghost"
        size="sm"
        className="ml-auto h-7 text-sm border border-white"
        onClick={onClear}
      >
        Clear selection
      </Button>
    </div>
  );
}
