/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import type { UserFacingIssue } from "@/types";
import { Modal } from "@/components/ui/modal";
import { Button } from "@/components/ui/button";
import { AlertTriangle, Info } from "lucide-react";

type Props = {
  issue: UserFacingIssue | null;
  onClose: () => void;
  onChoice?: (choiceId: string) => void;
};

export function IssueModal({ issue, onClose, onChoice }: Props) {
  if (!issue) return null;

  const isWarning = issue.id.includes("conflict") || issue.id.includes("missing") || issue.id.includes("req-");
  const Icon = isWarning ? AlertTriangle : Info;
  const iconColor = isWarning ? "text-[var(--warning)]" : "text-[var(--info)]";

  const handleChoice = (choiceId: string) => {
    onChoice?.(choiceId);
    if (choiceId === "acknowledge" || choiceId === "cancel" || choiceId === "got it") {
      onClose();
    }
  };

  return (
    <Modal
      open
      onOpenChange={(open) => !open && onClose()}
      title={issue.title}
      size="sm"
    >
      <div className="space-y-4 px-6 py-5">
        <div className="flex gap-3">
          <Icon size={22} className={`mt-0.5 shrink-0 ${iconColor}`} />
          <div className="min-w-0">
            <p className="text-sm leading-relaxed text-text-secondary">{issue.explanation}</p>
            <p className="mt-3 text-xs text-text-muted">{issue.impact}</p>
          </div>
        </div>
        <div className="flex flex-wrap justify-end gap-2 pt-2">
          {issue.choices.length > 0 ? (
            issue.choices.map((choice) => (
              <Button
                key={choice.id}
                variant={choice.recommended ? "accent" : "secondary"}
                size="sm"
                onClick={() => handleChoice(choice.id)}
                title={choice.description}
              >
                {choice.label}
              </Button>
            ))
          ) : (
            <Button variant="accent" size="sm" onClick={onClose}>
              Got it
            </Button>
          )}
        </div>
      </div>
    </Modal>
  );
}
