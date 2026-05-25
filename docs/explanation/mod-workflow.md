# Mod workflow

Supervisor separates **library state** (what mods you have and which are enabled) from **deployed state** (what files exist in the game folder).

## Staging

All mod archives extract into per-game staging under app data. Staging is the source of truth for mod files. Nothing touches the game directory until deploy.

Benefits:

- Disable a mod without re-downloading
- Multiple loadouts referencing the same staged files
- Safe rollback via purge + redeploy

## Loadouts

A loadout records:

- Which mod IDs are enabled
- Conflict resolutions (path → winning mod)
- Optional deploy path override

Switching loadouts changes the target deploy set without deleting staged mods.

## Deploy engine

Deploy creates **hardlinks** from staging into profile-specific paths (`Data/`, `Mods/`, etc.). Hardlinks share disk blocks — no duplicate copy of large assets.

Constraints:

- Staging and game must share a volume (Windows hardlink requirement)
- Purge removes previously deployed links before a fresh deploy (optional)
- Drift detection compares deployed manifest to on-disk files

## Conflicts

Conflicts occur when two enabled mods ship the same relative path. Supervisor stores the user’s choice in the loadout. Deploy applies only the winner.

## Game profiles

Profiles map a detected game to engine rules — mod folder, plugin support, prerequisite checks. See [Reference → Game profiles](../reference/game-profiles.md).

## Related

- [User → Mod management](../user/mod-management.md)
- [Reference → Game profiles](../reference/game-profiles.md)
