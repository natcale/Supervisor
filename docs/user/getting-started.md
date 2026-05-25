# Getting started

This tutorial walks through a first successful mod install on Windows.

## 1. Install Supervisor

Download the latest Windows installer from [GitHub Releases](https://github.com/natcale/Supervisor/releases). Supervisor targets **Windows 10/11** only.

## 2. Add a Nexus API key

Mod downloads from Nexus require a personal API key.

1. Open [nexusmods.com → My Account → API](https://www.nexusmods.com/users/myaccount?tab=api).
2. In Supervisor, go to **Settings → Nexus Mods**.
3. Paste your key and click **Test API key**.

The key is stored in Windows Credential Manager, not in plain text.

## 3. Scan or add a game

Supervisor detects games from Steam, Epic, GOG, and Heroic (Linux Heroic scan is not used on Windows).

1. Open **Settings → Games** and click **Refresh scan**, or
2. Use **Add local game** to point at any install folder.

Pick a game from **Library** to make it active. Enable **Remember last selected game** in Settings if you want Supervisor to restore your choice on launch.

## 4. Install a mod

**From Nexus (recommended)**

1. Set Supervisor as your Nexus “Mod Manager Download” app (easiest with a production build).
2. Click **Mod Manager Download** on a mod page.
3. Watch the download popup or open **Downloads** for progress.

**From a file**

1. Open **Mods** for your selected game.
2. Drop a `.zip` or `.7z` archive onto the mod list.
3. Complete the FOMOD wizard if prompted.

## 5. Deploy

Mods live in **staging** until deployed. Supervisor creates hardlinks into the game’s mod folder (for example `Data/` or `Mods/`).

1. Enable mods in the mod list.
2. Click **Deploy** (or enable **Auto-deploy when enabling or disabling mods** in Settings).

Staging and the game directory must be on the same drive for hardlinks.

## Next steps

- [Mod management](mod-management.md) — loadouts, conflicts, drift
- [Nexus downloads](nexus-downloads.md) — queue, clear, troubleshooting
- [Explanation → Mod workflow](../explanation/mod-workflow.md) — how the pieces fit together
