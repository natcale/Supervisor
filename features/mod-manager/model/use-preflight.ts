/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useEffect, useMemo, useState } from "react";
import { runPreflightChecks } from "@/lib/api/deploy";
import type { DetectedGame, DiagnosticReport, ModManifest } from "@/types";

type Options = {
  game?: DetectedGame;
  mods: ModManifest[];
  enabledIds: string[];
  stagingDir: string;
  profileId?: string;
  deployPathOverride?: string;
  conflictResolutions?: Record<string, string>;
  enabled?: boolean;
};

const emptyReport: DiagnosticReport = {
  ready: false,
  issues: [],
  summary: "Add mods to your queue.",
};

export function usePreflight({
  game,
  mods,
  enabledIds,
  stagingDir,
  profileId,
  deployPathOverride,
  conflictResolutions = {},
  enabled = true,
}: Options) {
  const needsScan =
    enabled && !!game && mods.length > 0 && enabledIds.length > 0 && Boolean(stagingDir);
  const [asyncReport, setAsyncReport] = useState<DiagnosticReport | null>(null);

  useEffect(() => {
    if (!needsScan || !game) {
      setAsyncReport(null);
      return;
    }

    let cancelled = false;
    runPreflightChecks(
      game.installPath,
      profileId ?? game.profileId,
      stagingDir,
      mods,
      enabledIds,
      conflictResolutions,
      deployPathOverride,
    ).then((result) => {
      if (!cancelled) setAsyncReport(result);
    });

    return () => {
      cancelled = true;
    };
  }, [game, mods, enabledIds, needsScan, profileId, deployPathOverride, stagingDir, conflictResolutions]);

  const report = useMemo(() => {
    if (!needsScan) return emptyReport;
    return asyncReport ?? emptyReport;
  }, [asyncReport, needsScan]);

  const loading = needsScan && !asyncReport;
  const blockingIssues = report.issues.filter(
    (i) => i.id.startsWith("conflict-") || i.id.startsWith("missing-dep-"),
  );
  const advisoryIssues = report.issues.filter(
    (i) =>
      i.id.startsWith("req-") ||
      i.id.startsWith("smf-") ||
      i.id.startsWith("profile-") ||
      i.id.startsWith("type-") ||
      i.id.startsWith("bsa-") ||
      i.id.startsWith("cp77-"),
  );

  return { report, loading, blockingIssues, advisoryIssues, ready: report.ready && !loading };
}
