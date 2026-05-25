/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
export interface ThemeSummary {
  id: string;
  name: string;
  author?: string;
  description?: string;
}

export interface ThemeSlotConfig {
  width?: number;
  density?: string;
  align?: "center" | "start" | "end";
  hidden?: boolean;
  /** When true, themes may show the compact game icon bar by default. */
  enabled?: boolean;
  showWalkthrough?: boolean;
  itemOrder?: string[];
}

export interface ThemeLayoutConfig {
  slots: Record<string, ThemeSlotConfig>;
}

export interface LoadedTheme {
  summary: ThemeSummary;
  css: {
    id: string;
    css: string;
    fontFaces: string;
  };
  layouts: ThemeLayoutConfig;
  fonts: Array<{
    family: string;
    weight: number;
    relativePath: string;
  }>;
}
