<!-- BEGIN:nextjs-agent-rules -->
# This is NOT the Next.js you know

This version has breaking changes — APIs, conventions, and file structure may all differ from your training data. Read the relevant guide in `node_modules/next/dist/docs/` before writing any code. Heed deprecation notices.
<!-- END:nextjs-agent-rules -->

# Supervisor

Supervisor is a cross-game mod manager and launcher for **Windows**. It detects installed games (Steam, Epic, GOG, Heroic), imports mods from archives or Nexus `nxm://` links, resolves conflicts, deploys via hardlinks, and launches games.

Tauri 2 (Rust backend) + Next.js 16 (React 19 frontend) + Tailwind CSS 4 + Bun.

### Architecture

- **Frontend → backend:** UI calls thin wrappers in `lib/api/*` or `lib/tauri.ts`, which invoke Tauri commands. Do not put domain logic in the frontend when it belongs in Rust.
- **Backend:** Keep Tauri commands thin. Put domain logic in `src-tauri/src/` modules (deploy, ingest, library, nexus, etc.).
- **Feature modules:** Colocate UI and hooks under `features/<name>/`. Prefer extending existing modules over creating parallel abstractions.
- **Events:** Tauri events (e.g. `download://completed`, `deploy://completed`) drive cross-panel updates in the shell.

## Development commands

```bash
bun install              # install dependencies
bun run tauri:dev        # run desktop app in dev mode
bun run lint             # ESLint
bun run build            # Next.js production build
bun run test:web         # Vitest frontend tests
bun run test             # Rust unit tests (cargo test)
bun run verify           # lint + build + test:web + test (full local CI)
bun run tauri:build      # build Windows installer
```

Rust formatting and checks (when touching `src-tauri/`):

```bash
cd src-tauri && cargo fmt --all
cd src-tauri && cargo check
```

## Agent rules

### Scope and style

- Make focused, minimal diffs. Match existing naming, imports, and patterns in the file you edit.
- Do not copy GPL code from Vortex or other mod managers — reimplement workflows under MIT.
- Do not commit secrets (`.env`, API keys, `src-tauri/updater.key`, credential files).
- Only create git commits or push when the user explicitly asks.
- Do not add markdown docs or tests unless requested or they meaningfully cover real behavior.

### Commit messages

When the user asks for a commit message (or you draft one for them to commit), use a **subject + body** format:

1. **Subject line** — one line, imperative mood, ≤72 characters. Prefix with a scope when helpful (`fix(mod-manager):`, `feat(deploy):`, `refactor(shell):`). Summarize the change, not the implementation detail.
2. **Blank line** after the subject.
3. **Body** — one or more paragraphs explaining:
   - **What was wrong or missing** (symptom, bad behavior, gap).
   - **Why it mattered** (user impact, correctness, safety).
   - **What the change does** (approach in plain language; avoid a file list unless small).

Wrap body lines at ~72 characters. Write in complete sentences. Do not use bullet lists unless several independent fixes belong in one commit.

**Example:**

```
fix(mod-manager): stabilize toolbar overflow and pointer-based load-order reorder

The mod toolbar used flex-wrap with items-stretch, which pushed action
buttons to the top when the row wrapped and broke layout at narrow widths.
Measure primary actions with a ResizeObserver and move overflow items into
a "more" menu so the bar stays on one aligned row.

Load-order dragging used HTML5 DnD on table rows, which is unreliable in
WebView2/Tauri: drops often never fired and reorder appeared to do nothing.
Replace it with pointer capture on the grip handle, fix downward-move splice
index math, and apply optimistic queue updates before persisting via
reorder_library_mods.
```

Match recent `git log` scope prefixes when present. Prefer one logical change per commit; if the user will split work, offer one message per commit.

### Frontend (Next.js / React)

- Read `node_modules/next/dist/docs/` before using Next.js APIs — this project uses Next.js 16 with breaking changes from older versions.
- Use `"use client"` only where needed. Prefer existing hooks and API wrappers over new abstractions.
- Respect React Compiler lint rules (`react-hooks/preserve-manual-memoization`, `react-hooks/immutability`). Declare functions with `useCallback` before effects that reference them.
- Tauri-specific UI should guard with `isTauri()` from `lib/env.ts` where the web build must degrade gracefully.

### Backend (Rust)

- Add or extend modules under `src-tauri/src/` rather than bloating `commands/`.
- Game-specific behavior goes in engine modules (`bethesda/`, `cyberpunk/`, `bg3/`, etc.) or game profiles — not scattered conditionals.
- Run `cargo test` after backend changes. Format with `cargo fmt`.

### Adding a game

1. Add a profile entry to `src-tauri/src/games/game_profiles.json`.
2. Extend `profile_loader.rs` if a new engine template is needed.
3. Run `cargo test` and confirm lookup tests pass for the new Steam app ID / Nexus domain.

See [CONTRIBUTING.md](CONTRIBUTING.md) and [docs/](docs/) for full details.

## Pre-commit / pre-push checks

**When the user asks to commit or push, always run checks first and fix any failures before proceeding.**

Minimum before commit:

```bash
bun run lint
```

Full verify before push (matches CI on `main`):

```bash
bun run verify
```

If Rust files changed, also run:

```bash
cd src-tauri && cargo fmt --all --check
cd src-tauri && cargo check
```

Do not commit or push while lint, build, or tests are failing. Report what was run and what passed or failed.

## CI reference

GitHub Actions (`.github/workflows/ci.yml`) runs on every push/PR to `main`:

1. `bun run lint`
2. `bun run test:web`
3. `bun run build`
4. `cargo test --locked` and `cargo check --locked`
5. Tauri NSIS build smoke test (unsigned)

Use `bun run verify` locally to catch most CI failures before pushing.
