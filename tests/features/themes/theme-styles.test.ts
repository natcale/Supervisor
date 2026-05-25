/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import { afterEach, describe, expect, it } from "vitest";
import {
  clearThemeStyles,
  FONT_STYLE_ID,
  injectStyles,
  STYLE_ID,
} from "@/features/themes/theme-styles";
import type { LoadedTheme } from "@/types";

function themeStub(partial: Partial<LoadedTheme> & { id?: string; css?: string }): LoadedTheme {
  return {
    summary: {
      id: partial.id ?? "test",
      name: "Test",
      version: "1.0.0",
      author: "",
      description: "",
    },
    css: { id: "main", css: partial.css ?? "", fontFaces: "" },
    fonts: [],
    layouts: { slots: {} },
  };
}

describe("theme styles", () => {
  afterEach(() => {
    clearThemeStyles();
  });

  it("injects CSS into a style element", () => {
    injectStyles(themeStub({ css: ":root { --primary: red; }" }), "");
    const style = document.getElementById(STYLE_ID);
    expect(style?.textContent).toContain("--primary");
  });

  it("clears injected theme styles", () => {
    injectStyles(themeStub({ css: "body { color: blue; }" }), " @font-face {} ");
    clearThemeStyles();
    expect(document.getElementById(STYLE_ID)).toBeNull();
    expect(document.getElementById(FONT_STYLE_ID)).toBeNull();
  });

  it("clears styles when default theme has no CSS", () => {
    injectStyles(themeStub({ id: "default", css: "body { color: red; }" }), "");
    injectStyles(themeStub({ id: "default", css: "" }), "");
    expect(document.getElementById(STYLE_ID)).toBeNull();
  });
});
