We appreciate your help improving Supervisor. Before opening a pull request, please read this guide.

## Guidelines

Supervisor is a desktop app (Rust + Next.js). Keep Tauri commands thin and put domain logic in `src-tauri/src/`.

### Before submitting

- Check for an existing PR or issue covering the same change.
- Run the full verify script locally:

```bash
bun run verify
```

This runs ESLint, the Next.js production build, Vitest frontend tests, and the Rust test suite.

### New features

Open an issue first if the feature changes mod-management behavior, game profile semantics, or deploy strategy. That keeps design aligned before code review.

### Bug fixes

Include steps to reproduce, expected vs actual behavior, and which game/profile was involved if relevant.

### Documentation

Docs live in `docs/` (organized by user guides, reference, explanation, and contributing). Update them when you change user-visible behavior, settings, game profile format, or the `.svtheme` theme system (see [docs/reference/themes/](docs/reference/themes/README.md)).

## Local development

### Prerequisites

- [Bun](https://bun.sh)
- [Rust](https://rustup.rs) (1.77+)
- [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS

### Setup

```bash
git clone https://github.com/natcale/supervisor.git
cd supervisor
bun install
bun run tauri:dev
```

### Formatting and linting

```bash
bun run lint
cd src-tauri && cargo fmt --all
```

### Type checking and build

```bash
bun run build
cd src-tauri && cargo check
```

### Tests

```bash
bun run test:web   # Vitest + React Testing Library
bun run test       # Rust unit tests
bun run verify     # All of the above + lint + build
```

Rust unit tests cover deep links, game profiles, detection filters, settings persistence, profile resolution, and theme validation. Frontend tests cover error formatting, settings merge defaults, theme CSS injection, and key UI helpers. See [docs/contributing/testing.md](docs/contributing/testing.md) for how to validate behavior without owning every game.

## Adding a game profile

1. Add an entry to `src-tauri/src/games/game_profiles.json` with `id`, `name`, `steamAppIds`, `nexusDomains`, and `engine` (or `modPath` for custom layouts).
2. If the game needs a new engine template, extend `profile_loader.rs` — do not copy GPL code from Vortex; reimplement the workflow under MIT.
3. Run `cargo test` and confirm lookup tests pass for the new Steam app ID and Nexus domain.

## Pull requests

Describe what changed and why. Confirm `bun run verify` passes. Prefer one focused change per PR.

## License

Contributions are licensed under the MIT License.
