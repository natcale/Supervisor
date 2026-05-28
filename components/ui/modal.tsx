/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import * as Dialog from "@radix-ui/react-dialog";
import { X } from "lucide-react";
import { cn } from "@/lib/cn";

type ModalProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title?: string;
  description?: string;
  children: React.ReactNode;
  className?: string;
  size?: "sm" | "md" | "lg";
};

const sizes = {
  sm: "max-w-md",
  md: "max-w-xl",
  lg: "max-w-2xl",
};

export function Modal({
  open,
  onOpenChange,
  title,
  description,
  children,
  className,
  size = "md",
}: ModalProps) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-x-0 bottom-0 top-[33px] z-modal bg-black/40" />
        <Dialog.Content
          className={cn(
            "fixed left-1/2 top-1/2 z-modal w-[calc(100%-2rem)] -translate-x-1/2 -translate-y-1/2",
            "rounded-2xl bg-panel-secondary shadow-md",
            "outline-none",
            sizes[size],
            className,
          )}
        >
          {(title || description) && (
            <div className="px-3 py-2">
              {title && (
                <Dialog.Title className="text-md font-regular text-text-active">
                  {title}
                </Dialog.Title>
              )}
              {description ? (
                <Dialog.Description className="mt-1 text-sm text-text-secondary">
                  {description}
                </Dialog.Description>
              ) : (
                <Dialog.Description className="sr-only">{title ?? "Dialog"}</Dialog.Description>
              )}
            </div>
          )}
          {!title && !description && (
            <Dialog.Description className="sr-only">Dialog</Dialog.Description>
          )}
          {children}
          <Dialog.Close
            className="absolute right-2 top-2 rounded-lg p-1 text-text-muted hover:bg-panel-hover hover:text-text-primary"
            aria-label="Close"
          >
            <X size={16} />
          </Dialog.Close>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
