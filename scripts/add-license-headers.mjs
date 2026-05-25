import fs from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "..");

const tsHeader = `/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
`;

const cssHeader = `/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
`;

const rustHeader = `/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
`;

const marker = "Copyright (c) Supervisor contributors";
const skipDirs = new Set(["node_modules", ".next", "out", "target", ".git", "icons"]);
const skipFiles = new Set(["next-env.d.ts", "bun.lock"]);

function walk(dir, files = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (skipDirs.has(entry.name)) continue;
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full, files);
    else files.push(full);
  }
  return files;
}

function headerFor(file) {
  if (file.endsWith(".rs")) return rustHeader;
  if (file.endsWith(".css")) return cssHeader;
  if (/\.(tsx?|jsx?|mjs)$/.test(file)) return tsHeader;
  return null;
}

let changed = 0;
for (const file of walk(root)) {
  const rel = path.relative(root, file);
  if (skipFiles.has(path.basename(file))) continue;
  if (rel.startsWith("scripts" + path.sep) && file.endsWith(".mjs")) continue;
  const header = headerFor(file);
  if (!header) continue;
  const content = fs.readFileSync(file, "utf8");
  if (content.includes(marker)) continue;
  fs.writeFileSync(file, header + (content.startsWith("#!") ? "\n" + content : content));
  changed++;
}
console.log(`Added headers to ${changed} files`);
