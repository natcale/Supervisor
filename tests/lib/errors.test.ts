/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import { beforeEach, describe, expect, it, vi } from "vitest";

const mockOpenPath = vi.fn();

vi.mock("@/lib/api/settings", () => ({
  openPath: (...args: unknown[]) => mockOpenPath(...args),
}));

import { formatInvokeError, openPathSafe } from "@/lib/errors";

describe("formatInvokeError", () => {
  it("returns Error message", () => {
    expect(formatInvokeError(new Error("boom"))).toBe("boom");
  });

  it("prefers explanation on user-facing issue shaped object", () => {
    expect(
      formatInvokeError({ title: "T", explanation: "The real message", choices: [], impact: "" }),
    ).toBe("The real message");
  });

  it("falls back to title without explanation", () => {
    expect(formatInvokeError({ title: "Only title", choices: [], impact: "" })).toBe("Only title");
  });

  it("returns plain string unchanged", () => {
    expect(formatInvokeError("plain")).toBe("plain");
  });

  it("returns fallback for unknown payloads", () => {
    expect(formatInvokeError(null)).toBe("Something went wrong.");
  });
});

describe("openPathSafe", () => {
  beforeEach(() => {
    mockOpenPath.mockReset();
  });

  it("returns null when open succeeds", async () => {
    mockOpenPath.mockResolvedValue(undefined);
    await expect(openPathSafe("C:\\fake")).resolves.toBeNull();
  });

  it("returns formatted explanation when invoke fails", async () => {
    mockOpenPath.mockRejectedValue({ explanation: "blocked" });
    await expect(openPathSafe("C:\\fake")).resolves.toBe("blocked");
  });
});
