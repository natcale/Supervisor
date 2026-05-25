/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import { describe, expect, it } from "vitest";
import { gameMatchesNxm } from "@/types";

describe("gameMatchesNxm", () => {
  it("matches on nexusDomain equality (case insensitive)", () => {
    expect(
      gameMatchesNxm(
        {
          id: "1",
          name: "Example",
          platform: "steam",
          installPath: "C:\\game",
          nexusDomain: "skyrimspecialedition",
        },
        "SkyrimSpecialEdition",
      ),
    ).toBe(true);
  });

  it("matches loosely on game name substring", () => {
    expect(
      gameMatchesNxm(
        {
          id: "1",
          name: "The Elder Scrolls V: Skyrim SE",
          platform: "steam",
          installPath: "C:\\game",
        },
        "skyrim",
      ),
    ).toBe(true);
  });

  it("returns false when no domain or substring match", () => {
    expect(
      gameMatchesNxm(
        {
          id: "1",
          name: "Stardew Valley",
          platform: "steam",
          installPath: "C:\\game",
          nexusDomain: "stardewvalley",
        },
        "fallout",
      ),
    ).toBe(false);
  });
});
