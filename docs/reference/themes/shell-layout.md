# Shell layout and theme slots

Supervisor’s shell has distinct regions. Themes can style any region via CSS; a subset also accepts JSON overrides.

The **compact game bar** (`shell.compactBar`) is optional and sits on the **far right** of the shell, it never replaces the main sidebar.

## Slot reference

| Slot ID | JSON fields | CSS hook |
|---------|-------------|----------|
| `shell.compactBar` | `enabled`, `hidden` | Yes |
| `shell.sidebar` | `width`, `density`, `hidden`, `itemOrder` | Yes |
| `shell.titlebar` | — | Yes |
| `shell.downloadPopup` | — | Yes |
| `shell.statusbar` | — | Yes |
| `view.mods.empty` | `align`, `showWalkthrough` | Yes |
| `view.downloads.empty` | `align`, `showWalkthrough` | Yes |
| `view.collections.vortex` | — | Yes (CSS only) |

### JSON field types

| Field | Type | Description |
|-------|------|-------------|
| `width` | number | Sidebar width in pixels |
| `density` | string | `"compact"` → smaller nav text |
| `align` | string | Empty state: `start`, `center`, `end` |
| `hidden` | boolean | Hide the region |
| `enabled` | boolean | Show compact game bar by default |
| `showWalkthrough` | boolean | Empty-state walkthrough link |
| `itemOrder` | string[] | Reorder main nav items |

Nav item ids: `home`, `downloads`, `games`, `settings`, `mods`, `collections`, `plugins`.

## Compact game bar

Enable via:

- **Settings → Compact game bar**, or
- Theme JSON: `"shell.compactBar": { "enabled": true }`

User settings override when you turn the bar off (`compactGameSidebarHidden`).

The bar shows game icons (Steam header art, cropped square), an add-game button, and a download trigger with popup.

## Example layout JSON

```json
{
  "slots": {
    "shell.compactBar": { "enabled": true },
    "shell.sidebar": {
      "width": 280,
      "density": "compact",
      "itemOrder": ["home", "mods", "downloads", "games", "settings"]
    },
    "view.mods.empty": { "align": "center", "showWalkthrough": true }
  }
}
```

## Adding slots (maintainers)

1. Add `data-theme-slot="region.name"` in React.
2. Document the slot in this file.
3. Optionally read fields via `useThemeLayout().getSlot()`.

## Related

- [CSS and slots](css-and-slots.md)
- [Package format](package-format.md)
