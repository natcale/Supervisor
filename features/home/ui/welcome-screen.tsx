/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import Image from "next/image";

export function WelcomeScreen() {
  return (
    <div className="flex h-full min-h-0 flex-col items-center justify-center gap-8 px-8 py-10 text-center">
      <Image
        src="/assets/logo/horizontal.svg"
        alt="Supervisor"
        width={360}
        height={64}
        className="h-auto w-full max-w-md"
        priority
      />
      <div className="space-y-2">
        <p className="text-xl text-text-secondary">
          Manage mods for your games and install, organize, deploy, and play.
        </p>
      </div>
    </div>
  );
}
