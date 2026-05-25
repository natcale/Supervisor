#!/usr/bin/env node
/**
 * Keeps package.json, Cargo.toml, and tauri.conf.json versions in sync.
 * Usage: node scripts/sync-version.mjs [version]
 * If version is omitted, reads from package.json.
 */
import { readFileSync, writeFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const pkgPath = join(root, "package.json");
const cargoPath = join(root, "src-tauri", "Cargo.toml");
const tauriPath = join(root, "src-tauri", "tauri.conf.json");

const pkg = JSON.parse(readFileSync(pkgPath, "utf8"));
const version = process.argv[2] ?? pkg.version;

if (!/^\d+\.\d+\.\d+(-[\w.]+)?$/.test(version)) {
  console.error(`Invalid semver: ${version}`);
  process.exit(1);
}

pkg.version = version;
writeFileSync(pkgPath, `${JSON.stringify(pkg, null, 2)}\n`);

let cargo = readFileSync(cargoPath, "utf8");
cargo = cargo.replace(/^version = ".*"$/m, `version = "${version}"`);
writeFileSync(cargoPath, cargo);

const tauri = JSON.parse(readFileSync(tauriPath, "utf8"));
tauri.version = version;
writeFileSync(tauriPath, `${JSON.stringify(tauri, null, 2)}\n`);

console.log(`Synced version to ${version}`);
