# Mod management

How to stage, enable, deploy, and maintain mods in Supervisor.

## Staging

Downloaded and imported mods live in per-game **staging** under app data (or a custom staging root in Settings). Files remain in staging until you deploy — deleting staging does not remove deployed hardlinks until you purge.

## Enable and disable

Toggle mods in the **Mods** tab. Disabled mods stay in your library but are not deployed.

Enable **Auto-deploy when enabling or disabling mods** in Settings if you want each toggle to redeploy immediately.

## Deploy

Deploy writes hardlinks from staging into the game’s mod path (defined by the [game profile](../reference/game-profiles.md)):

| Engine example | Typical folder |
|----------------|----------------|
| Bethesda | `Data/` |
| BepInEx | `BepInEx/plugins/` |
| Generic | `Data/` or profile-specific path |

**Requirements**

- Staging and game install on the **same drive/partition**
- Optional: **Purge before deploy** removes previous hardlinks first
- Optional: **Verify after deploy** checks linked files

## Conflicts

When two enabled mods contain the same relative path, Supervisor records a conflict. Pick the winning mod; the choice is stored in the active **loadout**.

## Loadouts

Each game has named loadouts — sets of enabled mods and conflict resolutions. Switch loadouts to change what gets deployed without removing mods from staging.

## Drift detection

If files in the game folder change outside Supervisor (manual edits, other tools), a drift warning appears. **Purge and redeploy** restores consistency with your loadout.

## Plugins (Bethesda)

For supported Bethesda profiles, the **Plugins** tab reads `plugins.txt`. Configure `loot.exe` in Settings for LOOT-based sorting.

## Related

- [Mod workflow (concepts)](../explanation/mod-workflow.md)
- [Game profiles](../reference/game-profiles.md)
