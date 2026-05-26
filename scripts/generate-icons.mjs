#!/usr/bin/env node
/**
 * Regenerate src-tauri/icons from public/logo.png (padded to square for Tauri CLI).
 * Requires: bun, tauri CLI, and on Windows uses PowerShell + System.Drawing.
 */
import { spawnSync } from "node:child_process";
import { existsSync, writeFileSync, unlinkSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { escapePowerShellSingleQuoted } from "./lib/ps-escape.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const logoPng = path.join(root, "public", "logo.png");
const logoIcon = path.join(root, "public", "logo-icon.png");
const iconsOut = path.join(root, "src-tauri", "icons");

if (!existsSync(logoPng)) {
  console.error("Missing public/logo.png");
  process.exit(1);
}

if (process.platform === "win32") {
  const psScript = path.join(root, "scripts", ".generate-icons-run.ps1");
  const ps = `
Add-Type -AssemblyName System.Drawing
$src = [System.Drawing.Image]::FromFile('${escapePowerShellSingleQuoted(logoPng)}')
$size = [Math]::Max($src.Width, $src.Height)
$bmp = New-Object System.Drawing.Bitmap $size, $size
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.Clear([System.Drawing.Color]::Transparent)
$x = [int](($size - $src.Width) / 2)
$y = [int](($size - $src.Height) / 2)
$g.DrawImage($src, $x, $y, $src.Width, $src.Height)
$bmp.Save('${escapePowerShellSingleQuoted(logoIcon)}', [System.Drawing.Imaging.ImageFormat]::Png)
$src.Dispose(); $bmp.Dispose(); $g.Dispose()
`;
  writeFileSync(psScript, ps, "utf8");
  const result = spawnSync(
    "powershell",
    ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", psScript],
    { stdio: "inherit", cwd: root },
  );
  unlinkSync(psScript);
  if (result.status !== 0) process.exit(result.status ?? 1);
} else if (existsSync(logoIcon)) {
  console.log("Using existing public/logo-icon.png (generate on Windows or add manually).");
} else {
  console.error("public/logo-icon.png not found. Run on Windows or create a square PNG from logo.png.");
  process.exit(1);
}

spawnSync("bun", ["run", "tauri", "icon", logoIcon, "-o", iconsOut], {
  stdio: "inherit",
  cwd: root,
});
console.log("Icons written to src-tauri/icons");
