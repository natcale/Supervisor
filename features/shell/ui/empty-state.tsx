/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import Image from "next/image";
import { open } from "@tauri-apps/plugin-shell";
import { useThemeLayout } from "@/features/themes/theme-provider";
import { isTauri } from "@/lib/env";

type LinkAction = {
  label: string;
  href: string;
};

type Props = {
  slotId?: string;
  iconSrc: string;
  iconWidth: number;
  iconHeight: number;
  title?: string;
  message: string;
  link?: LinkAction;
  align?: "center" | "start";
};

export function EmptyState({
  slotId,
  iconSrc,
  iconWidth,
  iconHeight,
  title,
  message,
  link,
  align = "center",
}: Props) {
  const { getSlot } = useThemeLayout();
  const slot = slotId ? getSlot(slotId) : undefined;
  const resolvedAlign = slot?.align ?? align;
  const centered = resolvedAlign === "center";
  const showLink = slot?.showWalkthrough !== false;

  const handleLink = () => {
    if (!link) return;
    if (isTauri()) {
      void open(link.href);
    } else {
      window.open(link.href, "_blank", "noopener,noreferrer");
    }
  };

  return (
    <div
      data-theme-slot={slotId}
      className={`flex flex-1 flex-col gap-4 p-8 ${
        centered ? "items-center justify-center text-center" : "items-start justify-center px-8 py-12"
      }`}
    >
      <Image
        src={iconSrc}
        alt=""
        width={iconWidth}
        height={iconHeight}
        className="shrink-0"
        aria-hidden
      />
      {title ? (
        <h2 className="text-lg font-medium text-text-primary">{title}</h2>
      ) : null}
      {message ? <p className="text-base text-text-primary">{message}</p> : null}
      {link && showLink ? (
        <button
          type="button"
          onClick={handleLink}
          className="text-sm text-text-primary underline underline-offset-2 hover:text-text-active"
        >
          {link.label}
        </button>
      ) : null}
    </div>
  );
}
