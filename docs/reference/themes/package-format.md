# Theme package format

## Archive layout

A valid `.svtheme` file is a ZIP archive:

```
theme.yaml
tokens.css
theme.css
layouts/
  shell.json
fonts/              # optional
  Inter.woff2
```

Subfolders such as `layouts/` must be preserved when packaging — do not flatten paths.

### Wrapper folders

If every file lives under one top-level directory (e.g. `my-theme/theme.yaml`), Supervisor strips that wrapper on install.

## theme.yaml

```yaml
id: my-theme
name: My Theme
author: You
description: Optional blurb
apiVersion: 1
minSupervisorVersion: "0.1.0"
css:
  - tokens.css
  - theme.css
layouts:
  - layouts/shell.json
fonts:
  - file: fonts/Inter.woff2
    family: Inter
    weight: 400
```

| Field | Required | Notes |
|-------|----------|-------|
| `id` | yes | Install folder name under app data |
| `name` | yes | Settings dropdown label |
| `author`, `description` | no | Metadata |
| `apiVersion` | no | Default `1` |
| `minSupervisorVersion` | no | Metadata hint (not enforced at runtime) |
| `css` | no | Relative paths, loaded in order |
| `layouts` | no | Merged slot JSON files |
| `fonts` | no | `.woff2` only |

## Install location

`%APPDATA%\com.supervisor.app\themes\<id>\`

Use **Open themes folder** in Settings to inspect or remove themes manually.

## Security (enforced in Rust)

| Rule | Limit |
|------|-------|
| CSS size | 512 KB max |
| Forbidden in CSS | `@import`, external URLs, `javascript:`, `<script>` |
| Fonts | `.woff2` only, 5 MB each |
| Forbidden in archive | `.js`, `.html`, `.exe`, `.bat`, `.cmd` |
| Path traversal | Blocked |

## Build

```bash
bun run package-theme my-theme
```

Output: `themes/my-theme.svtheme`

PowerShell alternative (from theme source directory):

```powershell
Compress-Archive -Path * -DestinationPath ../my-theme.svtheme -Force
```

## Default theme

Built-in **Default** (`id: default`) uses `app/globals.css` only. Missing or broken themes fall back to Default.

## Related

- [CSS and slots](css-and-slots.md)
- [Shell layout](shell-layout.md)
