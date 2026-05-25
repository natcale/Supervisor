/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useCallback, useEffect, useState } from "react";
import { Titlebar } from "@/features/layout/ui/titlebar";
import { Button } from "@/components/ui/button";
import { scanGames, completeOnboarding, getAppSettings } from "@/lib/tauri";
import { isOnboardingComplete, markOnboardingComplete } from "@/lib/onboarding";
import { isTauri } from "@/lib/env";
import { formatInvokeError } from "@/lib/errors";
import type { DetectedGame } from "@/types";
import { steamCoverUrl, withCover } from "@/types";
import { AlertTriangle } from "lucide-react";

type Step = "welcome" | "games" | "warning";

const STEPS: Step[] = ["welcome", "games", "warning"];

function platformLabel(platform: DetectedGame["platform"]) {
  switch (platform) {
    case "steam":
      return "Steam";
    case "epic":
      return "Epic";
    case "gog":
      return "GOG";
    case "heroic":
      return "Heroic";
    case "manual":
      return "Local";
    default:
      return platform;
  }
}

function GameRow({ game }: { game: DetectedGame }) {
  const enriched = withCover(game);
  const cover = enriched.coverUrl ?? steamCoverUrl(game.appId);
  const [broken, setBroken] = useState(false);

  return (
    <li className="flex shrink-0 items-center gap-3 rounded-lg border border-border bg-panel px-2 py-2">
      {cover && !broken ? (
        <img
          src={cover}
          alt=""
          className="aspect-[460/215] h-[52px] w-[112px] shrink-0 rounded object-cover"
          loading="lazy"
          onError={() => setBroken(true)}
        />
      ) : (
        <div className="flex aspect-[460/215] h-[52px] w-[112px] shrink-0 items-center justify-center rounded bg-panel-secondary text-xs font-medium text-text-muted">
          {game.name.slice(0, 2).toUpperCase()}
        </div>
      )}
      <div className="min-w-0 flex-1">
        <p className="truncate text-sm font-medium text-text-primary">{game.name}</p>
        <p className="truncate text-xs text-text-muted">{platformLabel(game.platform)}</p>
      </div>
    </li>
  );
}

export function OnboardingScreen() {
  const [step, setStep] = useState<Step>("welcome");
  const [games, setGames] = useState<DetectedGame[]>([]);
  const [showAll, setShowAll] = useState(false);
  const [loading, setLoading] = useState(false);
  const [busy, setBusy] = useState(false);
  const [checkingCompletion, setCheckingCompletion] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadGames = useCallback(async (includeAll: boolean) => {
    if (!isTauri()) return;
    setLoading(true);
    setError(null);
    try {
      const result = await scanGames({ includeAll });
      setGames(result.games);
    } catch (e) {
      setError(formatInvokeError(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (!isTauri()) {
      setCheckingCompletion(false);
      return;
    }
    if (isOnboardingComplete()) {
      void completeOnboarding().catch((e) => {
        setError(formatInvokeError(e));
        setCheckingCompletion(false);
      });
      return;
    }

    void getAppSettings()
      .then((settings) => {
        if (settings.onboardingComplete) {
          markOnboardingComplete();
          return completeOnboarding();
        }
        setCheckingCompletion(false);
      })
      .catch((e) => {
        setError(formatInvokeError(e));
        setCheckingCompletion(false);
      });
  }, []);

  useEffect(() => {
    if (step !== "games") return;
    void loadGames(showAll);
  }, [step, showAll, loadGames]);

  const stepIndex = STEPS.indexOf(step);

  const handleNext = () => {
    if (step === "warning") {
      void handleFinish();
      return;
    }
    setStep(STEPS[stepIndex + 1]!);
  };

  const handleBack = () => {
    if (stepIndex > 0) setStep(STEPS[stepIndex - 1]!);
  };

  const handleFinish = async () => {
    if (!isTauri()) return;
    setBusy(true);
    setError(null);
    try {
      markOnboardingComplete();
      await completeOnboarding();
    } catch (e) {
      setError(formatInvokeError(e));
      setBusy(false);
    }
  };

  if (checkingCompletion) {
    return (
      <div className="flex h-screen flex-col overflow-hidden bg-background">
        <Titlebar variant="compact" />
      </div>
    );
  }

  return (
    <div className="flex h-screen flex-col overflow-hidden bg-background">
      <Titlebar variant="compact" />
      <div className="flex min-h-0 flex-1 flex-col px-3 py-3">
        <div className="mb-4 flex items-center gap-2">
          {STEPS.map((id, i) => (
            <div
              key={id}
              className={`h-1.5 flex-1 rounded-full ${i <= stepIndex ? "bg-primary" : "bg-panel-secondary"}`}
            />
          ))}
        </div>

        <div className="min-h-0 flex-1">
          {step === "welcome" && (
            <div className="flex h-full flex-col justify-center gap-3">
              <h1 className="text-2xl font-regular text-text-active">Welcome to Supervisor</h1>
              <p className="text-md w-90 text-text-secondary">
                A mod manager for your installed games. This quick setup takes three steps
              </p>
            </div>
          )}

          {step === "games" && (
            <div className="flex h-full min-h-0 flex-col gap-3">
              <div>
                <h2 className="text-lg font-regular text-text-active">Your games</h2>
                <p className="mt-1 text-sm text-text-secondary">
                  We scanned Steam, Epic, GOG, Heroic, and any games you added manually.
                </p>
              </div>
              <label className="flex items-center gap-2 text-sm text-text-secondary">
                <input
                  type="checkbox"
                  checked={showAll}
                  onChange={(e) => setShowAll(e.target.checked)}
                  className="rounded border-border"
                />
                Show all installed titles (includes tools, SDKs, and non-game apps)
              </label>
              <div className="min-h-0 flex-1 overflow-y-auto">
                {loading ? (
                  <p className="p-3 text-sm text-text-muted">Scanning…</p>
                ) : games.length === 0 ? (
                  <p className="p-3 text-sm text-text-muted">
                    No games found. You can add them from Settings after setup.
                  </p>
                ) : (
                  <ul className="flex flex-col gap-2">
                    {games.map((game) => (
                      <GameRow key={game.id} game={game} />
                    ))}
                  </ul>
                )}
              </div>
              <p className="text-xs text-text-muted">{games.length} title(s) found</p>
            </div>
          )}

          {step === "warning" && (
            <div className="flex h-full flex-col justify-center gap-4">
              <section className="flex gap-3 p-3">
                <AlertTriangle size={22} className="mt-0.5 shrink-0 text-[var(--warning)]" />
                <div className="min-w-0 text-sm">
                  <p className="font-medium text-text-primary">Beta software</p>
                  <p className="mt-2 leading-relaxed text-text-secondary">
                    Supervisor 0.1.0 Beta is early software. Expect rough edges, missing features,
                    and occasional bugs. Back up saves and mod folders before deploying.
                  </p>
                </div>
              </section>
            </div>
          )}
        </div>

        {error && <p className="mt-3 text-sm text-[var(--danger)]">{error}</p>}

        <div className="mt-4 flex shrink-0 items-center justify-between gap-2 pt-4">
          <Button variant="ghost" size="sm" disabled={stepIndex === 0 || busy} onClick={handleBack}>
            Back
          </Button>
          <Button
            variant="accent"
            size="sm"
            disabled={busy || (step === "games" && loading)}
            onClick={() => void handleNext()}
          >
            {step === "warning" ? (busy ? "Opening…" : "Get started") : "Next"}
          </Button>
        </div>
      </div>
    </div>
  );
}
