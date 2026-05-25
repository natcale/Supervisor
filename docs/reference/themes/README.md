# Themes reference

Installable themes customize Supervisor via sanitized CSS and optional layout JSON. Themes do **not** run JavaScript or replace React components.

## Quick install

1. **Settings → Appearance → Install theme…** → select `.svtheme`
2. Choose the theme from the **Theme** dropdown
3. Select **Default** to remove injected CSS

Author reference (not preinstalled): `themes/packages/example/`

```bash
bun run package-theme example
```

## Reference pages

| Page | Contents |
|------|----------|
| [Package format](package-format.md) | `theme.yaml`, archive layout, security, build |
| [CSS and slots](css-and-slots.md) | Variables, `[data-theme-slot]` selectors |
| [Shell layout](shell-layout.md) | Dual sidebars, slot IDs, JSON fields |

## Concepts

[Explanation → Theme system](../../explanation/theme-system.md)

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| Theme missing from dropdown | Validation failed — reinstall; check `%APPDATA%\com.supervisor.app\themes\<id>\` |
| No visual change | Example theme CSS is commented out; uncomment rules or add `:root` overrides |
| Layout JSON ignored | Wrong slot id or unsupported field — see [Shell layout](shell-layout.md) |
| Compact bar won’t hide | Settings → Compact game bar |

Run `cargo test themes::tests` for archive and sanitization tests.
