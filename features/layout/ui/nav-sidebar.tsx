/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
"use client";

import type { DetectedGame, ShellView } from "@/types";
import { useThemeLayout } from "@/features/themes/theme-provider";
import {
  Download,
  Gamepad2,
  Home,
  Layers,
  Puzzle,
  Settings,
  Wrench,
} from "lucide-react";
import { SidebarGamePanel } from "@/features/layout/ui/sidebar-game-panel";

type Props = {
  active: ShellView;
  onNavigate: (view: ShellView) => void;
  game?: DetectedGame | null;
  modCount?: number;
  downloadActive?: number;
  supportsPlugins?: boolean;
  alwaysShowPlugins?: boolean;
};

const GENERAL: { id: ShellView; label: string; icon: typeof Download }[] = [
  { id: "home", label: "Home", icon: Home },
  { id: "downloads", label: "Downloads", icon: Download },
  { id: "games", label: "Library", icon: Gamepad2 },
  { id: "settings", label: "Settings", icon: Settings },
];

const MODS: { id: ShellView; label: string; icon: typeof Wrench }[] = [
  { id: "mods", label: "Mods", icon: Wrench },
  { id: "collections", label: "Collections", icon: Layers },
  { id: "plugins", label: "Plugins", icon: Puzzle },
];

function orderNavItems<T extends { id: string }>(items: T[], order?: string[]): T[] {
  if (!order?.length) return items;
  const map = new Map(items.map((item) => [item.id, item]));
  const ordered = order.map((id) => map.get(id)).filter((item): item is T => !!item);
  const rest = items.filter((item) => !order.includes(item.id));
  return [...ordered, ...rest];
}

export function NavSidebar({
  active,
  onNavigate,
  game = null,
  modCount = 0,
  downloadActive = 0,
  supportsPlugins = false,
  alwaysShowPlugins = false,
}: Props) {
  const { getSlot } = useThemeLayout();
  const sidebarSlot = getSlot("shell.sidebar");
  const hasGame = !!game;

  const generalItems = orderNavItems(GENERAL, sidebarSlot?.itemOrder);
  const modItems = orderNavItems(
    MODS.filter(({ id }) => id !== "plugins" || supportsPlugins || alwaysShowPlugins),
    sidebarSlot?.itemOrder,
  );

  const sidebarWidth = sidebarSlot?.width;
  const densityClass = sidebarSlot?.density === "compact" ? "text-xs" : "text-sm";

  if (sidebarSlot?.hidden) {
    return null;
  }

  return (
    <aside
      data-theme-slot="shell.sidebar"
      className={`flex h-full shrink-0 flex-col bg-sidebar px-1 ${densityClass} ${sidebarWidth ? "" : "w-nav"}`}
      style={sidebarWidth ? { width: sidebarWidth } : undefined}
    >
      <nav className="flex min-h-0 flex-1 flex-col overflow-y-auto py-2">
        <NavSection title="General">
          {generalItems.map(({ id, label, icon: Icon }) => (
            <NavItem
              key={id}
              label={label}
              icon={Icon}
              active={active === id}
              badge={id === "downloads" && downloadActive > 0 ? downloadActive : undefined}
              onClick={() => onNavigate(id)}
            />
          ))}
        </NavSection>

        <NavSection title="Mods" className="mt-3">
          {modItems.map(({ id, label, icon: Icon }) => (
            <NavItem
              key={id}
              label={label}
              icon={Icon}
              active={active === id}
              disabled={!hasGame}
              badge={id === "mods" && modCount > 0 ? modCount : undefined}
              onClick={() => hasGame && onNavigate(id)}
            />
          ))}
        </NavSection>
      </nav>

      <SidebarGamePanel game={game} onPickGame={() => onNavigate("games")} />
    </aside>
  );
}

function NavSection({
  title,
  children,
  className,
}: {
  title: string;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <div className={className}>
      <p className="mb-1 px-2 text-md font-medium text-text-muted">{title}</p>
      <ul className="flex flex-col gap-0.5">{children}</ul>
    </div>
  );
}

function NavItem({
  label,
  icon: Icon,
  active,
  disabled,
  badge,
  onClick,
}: {
  label: string;
  icon: typeof Download;
  active: boolean;
  disabled?: boolean;
  badge?: number;
  onClick: () => void;
}) {
  return (
    <li>
      <button
        type="button"
        disabled={disabled}
        onClick={onClick}
        className={`flex w-full items-center gap-2 rounded-lg px-2 py-1 text-lg disabled:cursor-not-allowed disabled:opacity-35 ${
          active
            ? "bg-panel-hover text-text-primary"
            : "text-text-secondary hover:bg-panel-hover hover:text-text-primary"
        }`}
      >
        <Icon size={16} className="shrink-0" />
        <span className="min-w-0 flex-1 truncate text-left">{label}</span>
        {badge !== undefined && badge > 0 && (
          <span className="shrink-0 rounded-full bg-primary px-2 text-sm text-white">{badge}</span>
        )}
      </button>
    </li>
  );
}
