# Deep links (NXM)

Nexus Mods sends download links via the `nxm://` protocol.

## URL shape

```
nxm://{domain}/mods/{modId}/files/{fileId}?key=…&expires=…&user_id=…
```

| Segment | Meaning |
|---------|---------|
| `domain` | Nexus game domain (e.g. `skyrimspecialedition`) |
| `modId` | Nexus mod ID |
| `fileId` | Nexus file ID |
| Query params | Optional NXM authorization (Mod Manager Download) |

OAuth callbacks use a separate payload shape; see `src-tauri/src/deep_link/parser.rs`.

## Processing pipeline

1. OS delivers the URL to Supervisor (`nxm://` handler)
2. Rust parses the URL (`deep_link/parser.rs`)
3. App emits `nxm://received` to the frontend
4. UI matches `domain` to an installed game’s Nexus domain
5. Backend enqueues download via Nexus API; completed archives ingest into staging

Duplicate NXM events within a short window are ignored.

## Windows single-instance

On Windows, a second process may start briefly, forward `argv` to the running instance, and exit. Supervisor raises the window only when minimized or hidden.

## API requirement

Supervisor resolves download URLs through the Nexus `download_link` API. A valid API key in Settings is required even when the NXM link includes a short-lived key.

## Troubleshooting

| Symptom | Likely cause |
|---------|----------------|
| Browser doesn’t offer Supervisor | `nxm://` not registered — run installer or dev build |
| App opens, no download | No installed game matches the domain |
| Download fails | Missing/invalid API key or expired NXM token |
| Wrong game | Multiple matches — select the correct game first |

## Testing

Unit tests: `deep_link/parser.rs`. End-to-end: real Nexus download with an installed game.

User guide: [Nexus downloads](../user/nexus-downloads.md)
