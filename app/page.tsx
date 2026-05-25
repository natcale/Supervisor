/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { ThemeProvider } from "@/features/themes/theme-provider";
import { App } from "@/features/shell/app";
import { DeveloperToolsShell } from "@/features/shell/developer-tools-shell";

export default function Home() {
  return (
    <ThemeProvider>
      <DeveloperToolsShell />
      <App />
    </ThemeProvider>
  );
}
