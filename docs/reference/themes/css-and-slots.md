# Theme CSS and slots

Themes restyle Supervisor primarily through **CSS custom properties** and **`[data-theme-slot]`** selectors injected at runtime.

## CSS variables

Override tokens from `app/globals.css` in `tokens.css`:

```css
:root {
  --background: #0f1419;
  --foreground: #e2e8f0;
  --panel: #151b23;
  --sidebar: #0d1117;
  --titlebar: #0d1117;
  --text-primary: #e6edf3;
  --text-secondary: #8b949e;
  --border: #30363d;
  --primary: #58a6ff;
  --radius-md: 8px;
  --nav-width: 280px;
  --font-ui: "Segoe UI", system-ui, sans-serif;
  --font-mono: "Cascadia Code", monospace;
}
```

### Common variables

| Variable | Used for |
|----------|----------|
| `--background`, `--foreground` | App base |
| `--panel`, `--panel-secondary`, `--content-panel` | Surfaces |
| `--sidebar`, `--titlebar`, `--statusbar` | Shell chrome |
| `--text-primary`, `--text-secondary`, `--text-muted` | Typography |
| `--border`, `--primary`, `--accent`, `--error`, `--success` | Accents |
| `--radius-sm`, `--radius-md`, `--radius-lg` | Corner radius |
| `--nav-width` | Main sidebar default width |
| `--font-ui`, `--font-mono` | Font stacks |

Components use `--font-ui`. Bundled `.woff2` fonts declared in `theme.yaml` inject `@font-face` rules at runtime.

## data-theme-slot selectors

Target stable regions in `theme.css`:

```css
[data-theme-slot="shell.titlebar"] {
  border-bottom: 1px solid var(--border);
}

[data-theme-slot="shell.sidebar"] button {
  border-radius: var(--radius-md);
}

[data-theme-slot="shell.compactBar"] {
  background: #0b0f14;
}

[data-theme-slot="shell.downloadPopup"] {
  box-shadow: 0 12px 40px rgb(0 0 0 / 45%);
}
```

Prefer slot selectors over brittle class names — slots are the supported extension surface.

Shared utility classes (e.g. `.bg-content-panel`) can be styled when no slot exists, but may change between app versions.

## Layout JSON (optional)

JSON can toggle a few layout fields (sidebar width, compact bar default, nav order). See [Shell layout](shell-layout.md).

Layout JSON does **not** replace CSS for visual styling.

## Example theme

`themes/packages/example/` contains commented patterns with **no active rules** until you uncomment them.

## Related

- [Package format](package-format.md)
- [Explanation → Theme system](../../explanation/theme-system.md)
