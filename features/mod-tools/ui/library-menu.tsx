/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useEffect, useMemo, useState } from "react";
import {
  getDeployState,
  getDeployTargets,
  purgeDeployedMods,
} from "@/lib/api/deploy";
import { getGameProfile } from "@/lib/api/games";
import type { DetectedGame, DeployTargetSummary, GameProfileSummary } from "@/types";
import { Button } from "@/components/ui/button";
import {
  Dropdown,
  DropdownContent,
  DropdownItem,
  DropdownLabel,
  DropdownSeparator,
  DropdownTrigger,
} from "@/components/ui/dropdown";
import { Check, MoreVertical } from "lucide-react";

type Props = {
  game: DetectedGame;
  deployPathOverride?: string;
  driftDetected?: boolean;
  onDeployPathOverride: (path: string | undefined) => void;
  onPurgeComplete: () => void;
  onDriftChecked: (drift: boolean) => void;
};

export function LibraryMenu({
  game,
  deployPathOverride,
  driftDetected,
  onDeployPathOverride,
  onPurgeComplete,
  onDriftChecked,
}: Props) {
  const [autoProfile, setAutoProfile] = useState<GameProfileSummary | null>(null);
  const [targets, setTargets] = useState<DeployTargetSummary[]>([]);
  const [purging, setPurging] = useState(false);
  const [installSummary, setInstallSummary] = useState<string | null>(null);

  useEffect(() => {
    getGameProfile(game).then(setAutoProfile).catch(console.error);
    getDeployTargets(game).then(setTargets).catch(console.error);
    getDeployState(game.id)
      .then((response) => {
        if (!response) return;
        const { state } = response;
        const outOfSync =
          !state.report.verified ||
          state.report.missing > 0 ||
          state.report.mismatched > 0;
        onDriftChecked(outOfSync || response.driftDetected);
        const when = new Date(state.manifest.deployedAt * 1000).toLocaleString();
        const status = state.report.verified
          ? `${state.report.linked} file(s) verified`
          : `${state.report.linked} linked, ${state.report.missing} missing${state.report.mismatched > 0 ? `, ${state.report.mismatched} mismatched` : ""}`;
        setInstallSummary(
          `${status} · ${when}${outOfSync || response.driftDetected ? " · drift detected" : ""}`,
        );
      })
      .catch(console.error);
  }, [game, onDriftChecked]);

  const activePath = useMemo(() => {
    if (deployPathOverride) return deployPathOverride;
    return autoProfile?.primaryModPath ?? "Data";
  }, [autoProfile, deployPathOverride]);

  const isGeneric = autoProfile?.isGeneric ?? false;
  const showTargetPicker = isGeneric && targets.length > 0;

  const purge = async () => {
    setPurging(true);
    try {
      await purgeDeployedMods(game.id);
      setInstallSummary(null);
      onPurgeComplete();
      onDriftChecked(false);
    } catch (e) {
      console.error(e);
    } finally {
      setPurging(false);
    }
  };

  return (
    <Dropdown>
      <DropdownTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          className="shrink-0 text-text-muted hover:text-text-primary"
          aria-label="Mod install settings"
        >
          <MoreVertical size={18} />
        </Button>
      </DropdownTrigger>
      <DropdownContent align="end" className="w-72">
        <DropdownLabel>Deploy target</DropdownLabel>
        <div className="px-2 pb-2 text-xs text-text-secondary">
          {autoProfile ? (
            <>
              <p className="font-medium text-text-primary">{autoProfile.name}</p>
              <p className="mt-0.5 font-mono text-text-muted">{activePath}</p>
              {!isGeneric && (
                <p className="mt-1 text-text-muted">Auto-detected — no change needed</p>
              )}
              {isGeneric && !deployPathOverride && (
                <p className="mt-1 text-warning">
                  Unknown game — pick a folder pattern below if Data/ is wrong.
                </p>
              )}
              {deployPathOverride && (
                <p className="mt-1 text-warning">Using custom folder pattern</p>
              )}
            </>
          ) : (
            <p>Detecting…</p>
          )}
        </div>

        {driftDetected && (
          <>
            <DropdownSeparator />
            <div className="px-2 pb-2 text-xs text-warning">
              Files in your game folder no longer match the last install. Reinstall or purge.
            </div>
          </>
        )}

        {installSummary && (
          <>
            <DropdownSeparator />
            <DropdownLabel>Last install</DropdownLabel>
            <p className="px-2 pb-2 text-xs text-text-secondary">{installSummary}</p>
          </>
        )}

        {showTargetPicker && (
          <>
            <DropdownSeparator />
            <DropdownLabel>Folder pattern</DropdownLabel>
            {targets.map((t) => {
              const selected = activePath === t.path;
              return (
                <DropdownItem key={t.id} onSelect={() => onDeployPathOverride(t.path)}>
                  <span className="flex-1 truncate">{t.label}</span>
                  {selected && <Check size={14} className="ml-2 shrink-0" />}
                </DropdownItem>
              );
            })}
            {deployPathOverride && (
              <DropdownItem onSelect={() => onDeployPathOverride(undefined)}>
                Reset to Data/
              </DropdownItem>
            )}
          </>
        )}

        {installSummary && (
          <>
            <DropdownSeparator />
            <DropdownItem disabled={purging} onSelect={() => void purge()}>
              {purging ? "Purging…" : "Purge installed mods"}
            </DropdownItem>
          </>
        )}
      </DropdownContent>
    </Dropdown>
  );
}
