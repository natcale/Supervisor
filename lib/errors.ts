/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
import type { UserFacingIssue } from "@/types";

export function formatInvokeError(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  if (error && typeof error === "object") {
    const issue = error as Partial<UserFacingIssue>;
    if (typeof issue.explanation === "string") return issue.explanation;
    if (typeof issue.title === "string") return issue.title;
  }
  return "Something went wrong.";
}

export async function openPathSafe(path: string): Promise<string | null> {
  const { openPath } = await import("@/lib/api/settings");
  try {
    await openPath(path);
    return null;
  } catch (error) {
    return formatInvokeError(error);
  }
}
