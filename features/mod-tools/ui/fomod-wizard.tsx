/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import { useEffect, useState } from "react";
import { Modal } from "@/components/ui/modal";
import { Button } from "@/components/ui/button";
import type { FomodConfig, FomodStep } from "@/types";

type Props = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  config: FomodConfig;
  modName: string;
  onComplete: (selections: string[]) => void;
  onInstallDefaults: () => void;
};

export function FomodWizard({
  open,
  onOpenChange,
  config,
  modName,
  onComplete,
  onInstallDefaults,
}: Props) {
  const [stepIndex, setStepIndex] = useState(0);
  const [selections, setSelections] = useState<Record<string, string>>({});

  useEffect(() => {
    if (!open) return;
    setStepIndex(0);
    const defaults: Record<string, string> = {};
    for (const step of config.steps) {
      const def = step.options.find((o) => o.isDefault) ?? step.options[0];
      if (def) defaults[step.id] = def.id;
    }
    setSelections(defaults);
  }, [open, config]);

  const step = config.steps[stepIndex];
  const isLast = stepIndex >= config.steps.length - 1;

  const pick = (stepId: string, optionId: string) => {
    setSelections((prev) => ({ ...prev, [stepId]: optionId }));
  };

  const finish = () => {
    onComplete(config.steps.map((s) => selections[s.id]).filter(Boolean));
    onOpenChange(false);
  };

  if (!step) return null;

  return (
    <Modal
      open={open}
      onOpenChange={onOpenChange}
      title={`FOMOD — ${modName}`}
      description={config.moduleName}
      size="md"
    >
      <StepView step={step} selected={selections[step.id]} onPick={(id) => pick(step.id, id)} />

      <div className="mt-6 flex items-center justify-between gap-2">
        <Button variant="ghost" size="sm" onClick={onInstallDefaults}>
          Use defaults
        </Button>
        <div className="flex gap-2">
          {stepIndex > 0 && (
            <Button variant="secondary" size="sm" onClick={() => setStepIndex((i) => i - 1)}>
              Back
            </Button>
          )}
          {!isLast ? (
            <Button
              variant="accent"
              size="sm"
              disabled={!selections[step.id]}
              onClick={() => setStepIndex((i) => i + 1)}
            >
              Next
            </Button>
          ) : (
            <Button variant="accent" size="sm" disabled={!selections[step.id]} onClick={finish}>
              Install
            </Button>
          )}
        </div>
      </div>

      {config.steps.length > 1 && (
        <p className="mt-3 text-xs text-text-muted">
          Step {stepIndex + 1} of {config.steps.length}: {step.name}
        </p>
      )}
    </Modal>
  );
}

function StepView({
  step,
  selected,
  onPick,
}: {
  step: FomodStep;
  selected?: string;
  onPick: (id: string) => void;
}) {
  return (
    <ul className="max-h-64 space-y-2 overflow-y-auto">
      {step.options.map((opt) => (
        <li key={opt.id}>
          <button
            type="button"
            onClick={() => onPick(opt.id)}
            className={`w-full rounded-md border px-3 py-2.5 text-left transition-colors ${
              selected === opt.id
                ? "border-[var(--accent)] bg-[var(--accent)]/10"
                : "border-border hover:bg-panel-hover"
            }`}
          >
            <p className="text-sm text-text-primary">{opt.name}</p>
            {opt.description && (
              <p className="mt-0.5 text-xs text-text-muted">{opt.description}</p>
            )}
          </button>
        </li>
      ))}
    </ul>
  );
}
