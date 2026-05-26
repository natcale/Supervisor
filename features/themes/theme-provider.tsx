/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { loadActiveTheme, readThemeAsset, setActiveTheme } from "@/lib/api/themes";
import { formatInvokeError } from "@/lib/errors";
import { isTauri } from "@/lib/env";
import { escapeCssSingleQuoted } from "@/lib/css-escape";
import { clearThemeStyles, injectStyles } from "@/features/themes/theme-styles";
import { listenSettingsChanged } from "@/lib/settings-events";
import { mergeSettings } from "@/lib/settings-defaults";
import type { LoadedTheme, ThemeSlotConfig, ThemeSummary } from "@/types";

type ThemeContextValue = {
  theme: LoadedTheme | null;
  loading: boolean;
  applyTheme: (themeId: string) => Promise<boolean>;
  getSlot: (slotId: string) => ThemeSlotConfig | undefined;
  refresh: () => Promise<void>;
};

const ThemeContext = createContext<ThemeContextValue | null>(null);

async function buildFontFaces(theme: LoadedTheme): Promise<string> {
  if (!isTauri() || theme.fonts.length === 0) return "";
  const faces: string[] = [];
  for (const font of theme.fonts) {
    const bytes = await readThemeAsset(theme.summary.id, font.relativePath);
    const blob = new Blob([new Uint8Array(bytes)], { type: "font/woff2" });
    const url = URL.createObjectURL(blob);
    faces.push(
      `@font-face { font-family: '${escapeCssSingleQuoted(font.family)}'; src: url('${url}') format('woff2'); font-weight: ${font.weight}; font-display: swap; }`,
    );
  }
  return faces.join("\n");
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setTheme] = useState<LoadedTheme | null>(null);
  const [loading, setLoading] = useState(true);

  const mountTheme = useCallback(async (loaded: LoadedTheme) => {
    const fontCss = await buildFontFaces(loaded);
    injectStyles(loaded, fontCss);
    setTheme(loaded);
  }, []);

  const refresh = useCallback(async () => {
    if (!isTauri()) {
      setTheme(null);
      setLoading(false);
      return;
    }
    setLoading(true);
    try {
      const loaded = await loadActiveTheme();
      await mountTheme(loaded);
    } catch (e) {
      console.warn(formatInvokeError(e));
      clearThemeStyles();
      setTheme(null);
    } finally {
      setLoading(false);
    }
  }, [mountTheme]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    if (!isTauri()) return;

    let unlisten: (() => void) | undefined;
    void listenSettingsChanged((settings) => {
      const next = mergeSettings(settings);
      const activeId = next.activeThemeId ?? "default";
      if (theme?.summary.id === activeId) return;
      void (async () => {
        try {
          const loaded = await setActiveTheme(activeId);
          await mountTheme(loaded);
        } catch (e) {
          console.warn(formatInvokeError(e));
        }
      })();
    }).then((fn) => {
      unlisten = fn;
    });

    return () => unlisten?.();
  }, [mountTheme, theme?.summary.id]);

  const applyTheme = useCallback(
    async (themeId: string): Promise<boolean> => {
      if (!isTauri()) return false;
      setLoading(true);
      try {
        const loaded = await setActiveTheme(themeId);
        await mountTheme(loaded);
        return true;
      } catch (e) {
        console.warn(formatInvokeError(e));
        if (themeId !== "default") {
          try {
            const loaded = await setActiveTheme("default");
            await mountTheme(loaded);
          } catch (fallbackError) {
            console.warn(formatInvokeError(fallbackError));
            clearThemeStyles();
            setTheme(null);
          }
        }
        return false;
      } finally {
        setLoading(false);
      }
    },
    [mountTheme],
  );

  const getSlot = useCallback(
    (slotId: string) => theme?.layouts.slots[slotId],
    [theme],
  );

  const value = useMemo(
    () => ({ theme, loading, applyTheme, getSlot, refresh }),
    [theme, loading, applyTheme, getSlot, refresh],
  );

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}

export function useThemeLayout() {
  const ctx = useContext(ThemeContext);
  if (!ctx) {
    return {
      theme: null as LoadedTheme | null,
      loading: false,
      applyTheme: async () => false,
      getSlot: () => undefined as ThemeSlotConfig | undefined,
      refresh: async () => {},
      summary: null as ThemeSummary | null,
    };
  }
  return { ...ctx, summary: ctx.theme?.summary ?? null };
}
