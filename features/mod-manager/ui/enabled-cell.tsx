/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { ChevronDown } from "lucide-react";

type Props = {
  enabled: boolean;
  disabled?: boolean;
  onToggle: () => void;
};

export function EnabledCell({ enabled, disabled, onToggle }: Props) {
  return (
    <button
      type="button"
      disabled={disabled}
      onClick={(e) => {
        e.stopPropagation();
        onToggle();
      }}
      className={`inline-flex items-center gap-1 rounded px-2 py-1 text-xs font-medium text-white disabled:opacity-50 ${
        enabled
          ? "bg-mod-enabled hover:bg-mod-enabled-hover"
          : "bg-[var(--button-secondary-bg)] text-text-secondary hover:bg-[var(--button-secondary-hover)]"
      }`}
    >
      {enabled ? "Enabled" : "Disabled"}
      <ChevronDown size={16} className="opacity-80" />
    </button>
  );
}
