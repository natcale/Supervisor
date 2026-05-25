/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import type { UserFacingIssue } from "@/types";
import { AlertTriangle, Info } from "lucide-react";

type Props = {
  issues: UserFacingIssue[];
  onReview: (issue: UserFacingIssue) => void;
};

export function DeployAdvisoryBanner({ issues, onReview }: Props) {
  if (issues.length === 0) return null;

  const blocking = issues.filter(
    (i) =>
      i.id.startsWith("req-") ||
      i.id.startsWith("missing-") ||
      i.id.startsWith("mismatch-"),
  );
  const informational = issues.filter(
    (i) =>
      !i.id.startsWith("req-") &&
      !i.id.startsWith("missing-") &&
      !i.id.startsWith("mismatch-"),
  );
  const primary = blocking[0] ?? informational[0];
  if (!primary) return null;

  const Icon = blocking.length > 0 ? AlertTriangle : Info;
  const tone = blocking.length > 0 ? "warning" : "info";

  return (
    <div
      className={
        tone === "warning"
          ? "shrink-0 border-b border-warning/30 bg-warning/10 px-3 py-2 text-xs text-warning"
          : "shrink-0 border-b border-info/30 bg-info/10 px-3 py-2 text-xs text-info"
      }
    >
      <div className="flex items-start gap-2">
        <Icon size={14} className="mt-0.5 shrink-0" />
        <div className="min-w-0 flex-1">
          <p className="font-medium">{primary.title}</p>
          <p className="mt-0.5 text-text-secondary">{primary.explanation}</p>
          {issues.length > 1 && (
            <p className="mt-1 text-text-muted">
              +{issues.length - 1} more deploy{" "}
              {issues.length === 2 ? "issue" : "issues"}
            </p>
          )}
        </div>
        <button
          type="button"
          className="shrink-0 underline"
          onClick={() => onReview(primary)}
        >
          Details
        </button>
      </div>
    </div>
  );
}
