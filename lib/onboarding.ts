/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

const ONBOARDING_KEY = "supervisor.onboarding.complete";

export function isOnboardingComplete(): boolean {
  if (typeof window === "undefined") return false;
  return window.localStorage.getItem(ONBOARDING_KEY) === "1";
}

export function markOnboardingComplete(): void {
  window.localStorage.setItem(ONBOARDING_KEY, "1");
}

export function clearOnboardingComplete(): void {
  window.localStorage.removeItem(ONBOARDING_KEY);
}
