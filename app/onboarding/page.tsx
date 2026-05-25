/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { OnboardingScreen } from "@/features/onboarding/onboarding-screen";
import { DeveloperToolsShell } from "@/features/shell/developer-tools-shell";

export default function OnboardingPage() {
  return (
    <>
      <DeveloperToolsShell />
      <OnboardingScreen />
    </>
  );
}
