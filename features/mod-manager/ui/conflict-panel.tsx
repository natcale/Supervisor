/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import type { UserFacingIssue } from "@/types";
import { AlertTriangle, X } from "lucide-react";

type Props = {
  issues: UserFacingIssue[];
  onResolve: (issueId: string, choiceId: string) => void;
  onDismiss: () => void;
};

export function ConflictPanel({ issues, onResolve, onDismiss }: Props) {
  const conflicts = issues.filter((i) => i.id.startsWith("conflict-"));
  const deps = issues.filter((i) => i.id.startsWith("missing-dep-"));

  if (conflicts.length === 0 && deps.length === 0) return null;

  return (
    <div className="shrink-0 border-b border-border bg-panel-secondary">
      <div className="flex items-center justify-between px-3 py-2">
        <div className="flex items-center gap-2 text-xs font-medium text-[var(--warning)]">
          <AlertTriangle size={14} />
          {conflicts.length + deps.length} issue{conflicts.length + deps.length === 1 ? "" : "s"} need attention
        </div>
        <button
          type="button"
          onClick={onDismiss}
          className="text-text-muted hover:text-text-primary"
        >
          <X size={14} />
        </button>
      </div>
      <ul className="max-h-40 overflow-y-auto px-3 pb-2">
        {[...conflicts, ...deps].map((issue) => (
          <li key={issue.id} className="mb-2 rounded border border-border bg-panel p-2 last:mb-0">
            <p className="text-xs font-medium text-text-primary">{issue.title}</p>
            <p className="mt-0.5 text-xs text-text-secondary">{issue.explanation}</p>
            {issue.choices.length > 0 && (
              <div className="mt-2 flex flex-wrap gap-1.5">
                {issue.choices
                  .filter((c) => c.id !== "cancel")
                  .map((choice) => (
                    <button
                      key={choice.id}
                      type="button"
                      title={choice.description}
                      onClick={() => onResolve(issue.id, choice.id)}
                      className={`rounded px-2 py-0.5 text-xs ${
                        choice.recommended
                          ? "bg-primary text-white"
                          : "bg-panel-hover text-text-secondary hover:text-text-primary"
                      }`}
                    >
                      {choice.label}
                    </button>
                  ))}
              </div>
            )}
          </li>
        ))}
      </ul>
    </div>
  );
}
