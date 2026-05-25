/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import { describe, expect, it } from "vitest";
import { mergeSettings, SETTINGS_DEFAULTS } from "@/lib/settings-defaults";
import type { AppSettings } from "@/types";

describe("SETTINGS_DEFAULTS", () => {
  it("defines core defaults expected by Settings UI", () => {
    expect(SETTINGS_DEFAULTS.rememberLastGame).toBe(true);
    expect(SETTINGS_DEFAULTS.updateCheckMode).toBe("onRefresh");
    expect(SETTINGS_DEFAULTS.activeThemeId).toBe("default");
    expect(SETTINGS_DEFAULTS.onboardingComplete).toBe(false);
  });
});

describe("mergeSettings", () => {
  it("fills missing keys from defaults while preserving overrides", () => {
    const raw = {
      compactModList: true,
      lastGameId: "g1",
      hasNexusApiKey: true,
    } as AppSettings;
    const merged = mergeSettings(raw);
    expect(merged.compactModList).toBe(true);
    expect(merged.lastGameId).toBe("g1");
    expect(merged.hasNexusApiKey).toBe(true);
    expect(merged.rememberLastGame).toBe(SETTINGS_DEFAULTS.rememberLastGame);
  });

  it("overrides defaults when backend sends patches", () => {
    const merged = mergeSettings({ ...SETTINGS_DEFAULTS, autoStartDownloads: false });
    expect(merged.autoStartDownloads).toBe(false);
  });
});
