/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { EmptyState } from "@/features/shell/ui/empty-state";

vi.mock("@/features/themes/theme-provider", () => ({
  useThemeLayout: () => ({
    getSlot: () => undefined,
  }),
}));

vi.mock("next/image", () => ({
  default: (props: { src: string; alt: string }) => <img src={props.src} alt={props.alt} />,
}));

vi.mock("@/lib/env", () => ({
  isTauri: () => false,
}));

describe("EmptyState", () => {
  const openSpy = vi.spyOn(window, "open");

  beforeEach(() => {
    openSpy.mockImplementation(() => null);
  });

  afterEach(() => {
    openSpy.mockRestore();
  });

  it("opens web link when not running in Tauri", () => {
    render(
      <EmptyState
        iconSrc="/x.svg"
        iconWidth={48}
        iconHeight={48}
        title="T"
        message="M"
        link={{ label: "Open", href: "https://example.com/path" }}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "Open" }));
    expect(openSpy).toHaveBeenCalledWith(
      "https://example.com/path",
      "_blank",
      "noopener,noreferrer",
    );
  });
});
