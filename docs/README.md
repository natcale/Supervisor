# Supervisor documentation

Supervisor is a Windows mod manager built with Tauri and Next.js. Documentation is organized by **what you need to do**, not by file history.

## Start here

| If you want to… | Go to |
|-----------------|-------|
| Install Supervisor and manage your first mod | [User guide → Getting started](user/getting-started.md) |
| Understand staging, deploy, and loadouts | [Explanation → Mod workflow](explanation/mod-workflow.md) |
| Look up NXM URLs, game profiles, or theme slots | [Reference](reference/README.md) |
| Build from source or run tests | [Contributing](contributing/README.md) |

## Documentation map

Documentation follows the [Diátaxis](https://diataxis.fr/) framework — four types of content, each with a different job:

| Type | Directory | Answers |
|------|-----------|---------|
| **Tutorials & how-to** | [`user/`](user/README.md) | “How do I accomplish X?” |
| **Reference** | [`reference/`](reference/README.md) | “What are the exact facts?” |
| **Explanation** | [`explanation/`](explanation/README.md) | “Why does it work this way?” |
| **Contributing** | [`contributing/`](contributing/README.md) | “How do I develop or test Supervisor?” |

## By audience

### End users

- [Getting started](user/getting-started.md) — install, add games, first mod
- [Mod management](user/mod-management.md) — staging, deploy, conflicts, loadouts
- [Nexus downloads](user/nexus-downloads.md) — API key, Mod Manager Download, queue

### Theme authors

- [Theme reference](reference/themes/README.md) — `.svtheme` format, CSS variables, layout slots
- [Theme system (concepts)](explanation/theme-system.md) — how injection and slots work
- Example source: `themes/packages/example/` (not bundled with the app)

### Contributors & maintainers

- [Local development](contributing/local-development.md)
- [Testing guide](contributing/testing.md)
- [CONTRIBUTING.md](../CONTRIBUTING.md) — game profiles, PR expectations

## Related files (repo root)

| File | Purpose |
|------|---------|
| [README.md](../README.md) | Project overview and quick install |
| [CONTRIBUTING.md](../CONTRIBUTING.md) | Contribution workflow |
| [PRIVACY.md](../PRIVACY.md) | Data handling |
| [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) | Attribution |
