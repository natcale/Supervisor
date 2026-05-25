# Testing guide

You do not need every supported game installed to validate Supervisor. Use automated tests, manual folders, and one or two real games.

## Automated tests

Full suite:

```bash
bun run verify
```

Individual targets:

```bash
bun run test:web   # Vitest + React Testing Library
bun run test       # Rust tests
```

**Frontend:** error formatting, settings defaults, theme CSS injection, empty state UI, NXM domain matching, games table.

**Rust:** NXM parsing, game profiles, profile resolution, detection filters, settings persistence, theme validation, FOMOD/VDF parsers.

CI runs the same on every push and PR.

## Manual testing

### Manual game folders

**Settings → Games → Add local game** attaches a profile when possible or falls back to generic `Data/` deploy.

Smoke-test tree:

```
C:\TestGames\FakeRPG\
  Data\
```

Drop a zip mod with a `Data/` root, deploy, verify hardlinks.

### NXM download flow

1. Nexus API key in Settings → **Test API key**
2. Register Supervisor as Nexus mod manager (production build recommended)
3. Mod Manager Download on a free mod for an installed game
4. Confirm download popup / Downloads page shows one queue entry

### Deploy and drift

1. Install mod, deploy
2. Delete a deployed file in the game folder
3. Reopen Mods — drift warning should appear

### Settings and scan

Disable Steam scan, refresh — Steam titles disappear; manual entries remain.

### Themes

1. `bun run package-theme example`
2. Settings → Appearance → Install theme…
3. Select **Example** — compact bar slot applies; CSS is commented (no color change until edited)
4. Switch to **Default** — fallback works

See [Reference → Themes](../reference/themes/README.md).

### App auto-updates

Release build only: Settings → General → **Check for app updates**.

## Validation matrix

| Layer | Pass criteria |
|-------|---------------|
| Detection | Game appears after scan or manual add |
| Profile | Settings lists correct mod path |
| Install | Archive extracts to staging |
| Deploy | Hardlinks in game folder; drift check passes |
| NXM | Download completes and ingests |

One real game validates the full pipeline. Profile list accuracy is guarded by `cargo test` lookup tests.

## Reporting issues

Include: OS, Supervisor version, game name, profile id, steps, platform (Steam/Epic/manual).

## Related

- [Local development](local-development.md)
- [User → Nexus downloads](../user/nexus-downloads.md)
