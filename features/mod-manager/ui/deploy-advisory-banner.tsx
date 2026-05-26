/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import type { UserFacingIssue } from "@/types";

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

  const tone = blocking.length > 0 ? "warning" : "info";

  return (
    <div
      className={
        tone === "warning"
          ? "shrink-0 border-b border-border px-2 py-1.5 text-sm text-warning"
          : "shrink-0 border-b border-border px-2 py-1.5 text-sm text-text-primary"
      }
    >
      <div className="flex items-start gap-2">
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
