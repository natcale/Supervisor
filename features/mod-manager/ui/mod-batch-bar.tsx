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
    <div className="flex shrink-0 items-center gap-2 bg-primary/20 px-5 py-2 text-sm">
      <span className="text-text-primary">{count} selected</span>
      <Button variant="secondary" size="sm" className="h-7 text-xs" onClick={onEnableAll}>
        Enable
      </Button>
      <Button variant="secondary" size="sm" className="h-7 text-xs" onClick={onDisableAll}>
        Disable
      </Button>
      <Button variant="secondary" size="sm" className="h-7 text-xs" onClick={onRemoveAll}>
        Remove
      </Button>
      <Button variant="ghost" size="sm" className="ml-auto h-7 text-xs" onClick={onClear}>
        Clear selection
      </Button>
    </div>
  );
}
