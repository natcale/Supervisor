import fs from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "..");

const replacements = [
  ["bg-[var(--background)]", "bg-background"],
  ["bg-[var(--panel-secondary)]", "bg-panel-secondary"],
  ["bg-[var(--panel-hover)]", "bg-panel-hover"],
  ["bg-[var(--panel-active)]", "bg-panel-active"],
  ["bg-[var(--content-panel)]", "bg-content-panel"],
  ["bg-[var(--sidebar)]", "bg-sidebar"],
  ["bg-[var(--primary)]", "bg-primary"],
  ["bg-[var(--toolbar-bg)]", "bg-toolbar"],
  ["bg-[var(--mod-enabled)]", "bg-mod-enabled"],
  ["bg-[var(--mod-enabled-hover)]", "bg-mod-enabled-hover"],
  ["bg-[var(--table-header)]", "bg-table-header"],
  ["bg-[var(--table-row-hover)]", "bg-table-row-hover"],
  ["bg-[var(--panel)]", "bg-panel"],
  ["bg-[var(--card)]", "bg-card"],
  ["bg-[var(--titlebar)]", "bg-titlebar"],
  ["bg-[var(--statusbar)]", "bg-statusbar"],
  ["bg-[var(--input-bg)]", "bg-input-bg"],
  ["bg-[var(--button-bg)]", "bg-button-bg"],
  ["text-[var(--text-primary)]", "text-text-primary"],
  ["text-[var(--text-secondary)]", "text-text-secondary"],
  ["text-[var(--text-muted)]", "text-text-muted"],
  ["text-[var(--text-active)]", "text-text-active"],
  ["text-[var(--text-disabled)]", "text-text-disabled"],
  ["text-[var(--primary)]", "text-primary"],
  ["text-[var(--statusbar-fg)]", "text-statusbar-fg"],
  ["border-[var(--border)]", "border-border"],
  ["border-[var(--primary)]", "border-primary"],
  ["border-[var(--border-subtle)]", "border-border"],
  ["rounded-[var(--radius-md)]", "rounded-md"],
  ["rounded-[var(--radius-lg)]", "rounded-lg"],
  ["rounded-[var(--radius-sm)]", "rounded-sm"],
  ["w-[var(--nav-width)]", "w-nav"],
  ["z-[var(--z-modal)]", "z-modal"],
  ["z-[var(--z-dropdown)]", "z-dropdown"],
  ["hover:bg-[var(--panel-hover)]", "hover:bg-panel-hover"],
  ["hover:bg-[var(--primary-hover)]", "hover:bg-primary-hover"],
  ["hover:bg-[var(--mod-enabled-hover)]", "hover:bg-mod-enabled-hover"],
  ["hover:text-[var(--text-primary)]", "hover:text-text-primary"],
  ["from-[var(--content-panel)]", "from-content-panel"],
  ["via-[var(--content-panel)]/70", "via-content-panel/70"],
  ["ring-[var(--border-focus)]", "ring-border-focus"],
  ["focus:border-[var(--border-focus)]", "focus:border-border-focus"],
  ["placeholder:text-[var(--input-placeholder)]", "placeholder:text-text-muted"],
];

const skipDirs = new Set(["node_modules", ".next", "out", "target", ".git"]);

function walk(dir, files = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (skipDirs.has(entry.name)) continue;
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full, files);
    else if (/\.(tsx?|jsx?)$/.test(entry.name)) files.push(full);
  }
  return files;
}

let changed = 0;
for (const file of walk(root)) {
  let content = fs.readFileSync(file, "utf8");
  let next = content;
  for (const [from, to] of replacements) {
    next = next.split(from).join(to);
  }
  if (next !== content) {
    fs.writeFileSync(file, next);
    changed++;
  }
}
console.log(`Updated ${changed} files`);
