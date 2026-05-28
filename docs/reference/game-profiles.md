# Game profiles

Game profiles define deploy paths, engine behavior, and detection metadata.

**Source file:** `src-tauri/src/games/game_profiles.json`

## Profile fields

| Field | Purpose |
|-------|---------|
| `id` | Internal profile key |
| `name` | Display name |
| `steamAppIds` | Steam app IDs for auto-detection |
| `nexusDomains` | Nexus site domains for NXM matching |
| `engine` | Template expanding to mod types and requirements |

Unknown games fall back to `generic-data` (`Data/` deploy).

## Engine templates

| Engine | Typical games | Mod folder |
|--------|---------------|------------|
| `bethesda` | Skyrim, Fallout | `Data/` + plugins |
| `bepinex` | Valheim, Lethal Company | `BepInEx/plugins/` |
| `mods` | Witcher 3, RimWorld | `Mods/` |
| `mod_root` | Elden Ring, Sekiro | `mod/` |
| `cyberpunk` | Cyberpunk 2077 | `archive/pc/mod` |
| `unreal_pak` | UE games with `.pak` | `…/Paks/~mods` |
| `mod_path` | Custom | `modPath` field |

## Adding a profile

See [CONTRIBUTING.md](../../CONTRIBUTING.md). After editing JSON, run `cargo test` — profile loader tests validate structure and lookups.

## Viewing built-in profiles

**Settings** lists every profile with mod path and plugin support.

## Related

- [Mod management](../user/mod-management.md)
- [Explanation → Mod workflow](../explanation/mod-workflow.md)
