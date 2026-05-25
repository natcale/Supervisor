/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { GamesTable } from "@/features/game-library/ui/games-table";
import type { DetectedGame } from "@/types";

const sampleGame: DetectedGame = {
  id: "steam-489830",
  name: "Skyrim SE",
  platform: "steam",
  installPath: "C:\\Games\\Skyrim",
  appId: "489830",
  nexusDomain: "skyrimspecialedition",
};

describe("GamesTable", () => {
  it("applies maxListHeight to the scroll container", () => {
    const { container } = render(
      <GamesTable
        games={[sampleGame]}
        maxListHeight="20rem"
        onSelect={vi.fn()}
        onRefresh={vi.fn()}
        onAddLocal={vi.fn()}
        onRemove={vi.fn()}
        onOpenFolder={vi.fn()}
      />,
    );
    const scroll = container.querySelector(".overflow-auto");
    expect(scroll).toHaveStyle({ maxHeight: "20rem" });
  });

  it("shows Nexus domain for detected games", () => {
    render(
      <GamesTable
        games={[sampleGame]}
        onSelect={vi.fn()}
        onRefresh={vi.fn()}
        onAddLocal={vi.fn()}
        onRemove={vi.fn()}
        onOpenFolder={vi.fn()}
      />,
    );
    expect(screen.getAllByText("skyrimspecialedition").length).toBeGreaterThan(0);
  });
});
