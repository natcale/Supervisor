# Nexus downloads

How to download mods from Nexus Mods into Supervisor.

## Prerequisites

1. **Nexus API key** — Settings → Nexus Mods → paste key → **Test API key**
2. **Matching game installed** — the NXM link’s domain must match a game in your library
3. **Protocol registration** — run a production build or dev build once so Windows registers `nxm://`

## Mod Manager Download

1. On Nexus, click **Mod Manager Download** for a file.
2. Your browser opens an `nxm://` URL; Windows routes it to Supervisor.
3. Supervisor queues the download, fetches the file via the Nexus API, and ingests it into staging.

Open the **download popup** from the titlebar or compact game bar for a live queue. Use **Downloads** in the sidebar for the full list with mod thumbnails.

## Download queue

| Status | Meaning |
|--------|---------|
| queued | Waiting to start |
| downloading | Transfer in progress |
| installing | Archive extracted and ingested |
| failed | Error — see message; use **Clear failed** in Settings |

Completed jobs are removed from the queue after ingest. Use **Clear finished** to remove failed or cancelled entries.

Settings under **Downloads**:

- **Max concurrent downloads** (1–6)
- **Auto-start queued downloads**
- **Speed limit** (KB/s, 0 = unlimited)

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| Browser doesn’t offer Supervisor | Re-run installer or `tauri:dev` to register `nxm://` |
| No download after link opens | Add a game whose Nexus domain matches the mod |
| Download fails | Test API key; retry Mod Manager Download for a fresh NXM token |
| Duplicate queue entries | Clear finished downloads; only one active job per mod file is kept |

Technical details: [Reference → Deep links](../reference/deep-links.md)
