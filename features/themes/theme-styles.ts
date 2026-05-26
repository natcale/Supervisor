/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import { escapeCssSingleQuoted } from "@/lib/css-escape";
import type { LoadedTheme } from "@/types";

export const STYLE_ID = "supervisor-theme-css";
export const FONT_STYLE_ID = "supervisor-theme-fonts";

export function clearThemeStyles() {
  document.getElementById(STYLE_ID)?.remove();
  document.getElementById(FONT_STYLE_ID)?.remove();
  document.documentElement.style.removeProperty("--font-sans");
}

export function injectStyles(theme: LoadedTheme, fontCss: string) {
  if (theme.summary.id === "default" && !theme.css.css && !fontCss) {
    clearThemeStyles();
    return;
  }
  let style = document.getElementById(STYLE_ID);
  if (!style) {
    style = document.createElement("style");
    style.id = STYLE_ID;
    document.head.appendChild(style);
  }
  style.textContent = theme.css.css;

  let fontStyle = document.getElementById(FONT_STYLE_ID);
  if (fontCss) {
    if (!fontStyle) {
      fontStyle = document.createElement("style");
      fontStyle.id = FONT_STYLE_ID;
      document.head.appendChild(fontStyle);
    }
    fontStyle.textContent = fontCss;
  } else if (fontStyle) {
    fontStyle.remove();
  }

  if (theme.css.css.includes("--font-sans") || theme.fonts.length > 0) {
    const primary = theme.fonts[0]?.family;
    if (primary) {
      document.documentElement.style.setProperty(
        "--font-sans",
        `'${escapeCssSingleQuoted(primary)}', system-ui, sans-serif`,
      );
    }
  }
}
