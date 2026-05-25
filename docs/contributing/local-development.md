# Local development

## Prerequisites

Install Bun, Rust, and the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for Windows.

## Run in dev mode

```bash
bun install
bun run tauri:dev
```

Starts the Next.js dev server and Tauri shell with frontend hot reload.

## Commands

| Command | Purpose |
|---------|---------|
| `bun run tauri:dev` | Desktop app in development |
| `bun run build` | Production frontend build |
| `bun run lint` | ESLint |
| `bun run test` | Rust unit tests |
| `bun run test:web` | Vitest frontend tests |
| `bun run verify` | Lint + build + frontend tests + Rust tests (CI equivalent) |
| `cd src-tauri && cargo check` | Fast Rust compile check |
| `cd src-tauri && cargo fmt --all` | Format Rust |

## Repository layout

| Path | Purpose |
|------|---------|
| `app/` | Next.js app router |
| `features/` | React UI by domain (shell, mod manager, settings, themes) |
| `lib/` | Tauri invoke wrappers |
| `themes/packages/` | Author reference themes (`example/`) |
| `src-tauri/src/commands/` | Thin Tauri handlers |
| `src-tauri/src/games/` | Game profiles |
| `src-tauri/src/themes/` | Theme install and CSS sanitizer |
| `src-tauri/src/deploy/` | Hardlink deploy engine |
| `src-tauri/src/game_detection/` | Steam/Epic/GOG/Heroic/manual scan |
| `docs/` | Documentation (see [docs hub](../README.md)) |

## Releases

Windows installers and portable ZIPs build on version tags (`v*`). Artifacts:

- x64 and ARM64 NSIS installers
- x64 and ARM64 portable ZIPs
- `latest.json` for in-app updates (ed25519 signatures)

See [GitHub Releases](https://github.com/natcale/Supervisor/releases). Only **Windows** is supported.

**Maintainers only:** auto-update signing uses a Tauri keypair (`bun run tauri signer generate -w src-tauri/updater.key`). Put the private key in GitHub Actions as `TAURI_SIGNING_PRIVATE_KEY` (and password secret if used); the public key lives in `src-tauri/tauri.windows.conf.json`. This is not Authenticode — SmartScreen warnings on the installer are expected.

Dev builds register `nxm://` when the app runs. Use a production build to test protocol handling like end users.

## Related

- [Testing guide](testing.md)
- [CONTRIBUTING.md](../../CONTRIBUTING.md)
