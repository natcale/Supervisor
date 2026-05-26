#!/usr/bin/env node
/**
 * Package themes/packages/<id>/ into themes/<id>.svtheme and public/assets/themes/<id>.svtheme
 */
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const id = process.argv[2];
if (!id || !/^[a-z0-9][a-z0-9-]*$/i.test(id)) {
  console.error("Usage: node scripts/package-theme.mjs <theme-id> (alphanumeric and hyphens only)");
  process.exit(1);
}

const src = path.join(root, "themes", "packages", id);
if (!fs.existsSync(path.join(src, "theme.yaml"))) {
  console.error(`Missing theme.yaml in ${src}`);
  process.exit(1);
}

const targets = [
  path.join(root, "themes", `${id}.svtheme`),
  path.join(root, "public", "assets", "themes", `${id}.svtheme`),
];

for (const out of targets) {
  fs.mkdirSync(path.dirname(out), { recursive: true });
  const tmpZip = `${out}.zip`;
  if (fs.existsSync(out)) fs.unlinkSync(out);
  if (fs.existsSync(tmpZip)) fs.unlinkSync(tmpZip);
  if (process.platform === "win32") {
    const archivePath = path.join(src, "*");
    const result = spawnSync(
      "powershell",
      [
        "-NoProfile",
        "-Command",
        "Compress-Archive",
        "-Path",
        archivePath,
        "-DestinationPath",
        tmpZip,
        "-Force",
      ],
      { stdio: "inherit", cwd: root },
    );
    if (result.status !== 0) process.exit(result.status ?? 1);
  } else {
    const result = spawnSync("zip", ["-r", tmpZip, "."], { stdio: "inherit", cwd: src });
    if (result.status !== 0) process.exit(result.status ?? 1);
  }
  fs.renameSync(tmpZip, out);
  console.log(`Wrote ${path.relative(root, out)}`);
}
