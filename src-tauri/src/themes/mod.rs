/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

const MAX_CSS_BYTES: usize = 512 * 1024;
const MAX_FONT_BYTES: usize = 5 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeFontDef {
    pub file: String,
    pub family: String,
    #[serde(default = "default_weight")]
    pub weight: u16,
}

fn default_weight() -> u16 {
    400
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeManifest {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_api_version")]
    pub api_version: u32,
    #[serde(default)]
    pub min_supervisor_version: Option<String>,
    #[serde(default)]
    pub fonts: Vec<ThemeFontDef>,
    #[serde(default)]
    pub css: Vec<String>,
    #[serde(default)]
    pub layouts: Vec<String>,
}

fn default_api_version() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeSummary {
    pub id: String,
    pub name: String,
    pub author: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeCssBundle {
    pub id: String,
    pub css: String,
    pub font_faces: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ThemeSlotConfig {
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub density: Option<String>,
    #[serde(default)]
    pub align: Option<String>,
    #[serde(default)]
    pub hidden: Option<bool>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub show_walkthrough: Option<bool>,
    #[serde(default)]
    pub item_order: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeLayoutConfig {
    #[serde(default)]
    pub slots: std::collections::HashMap<String, ThemeSlotConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeFontAsset {
    pub family: String,
    pub weight: u16,
    pub relative_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadedTheme {
    pub summary: ThemeSummary,
    pub css: ThemeCssBundle,
    pub layouts: ThemeLayoutConfig,
    pub fonts: Vec<ThemeFontAsset>,
}

pub fn themes_dir(app_data: &Path) -> std::path::PathBuf {
    app_data.join("themes")
}

pub fn ensure_themes_dir(app_data: &Path) -> AppResult<std::path::PathBuf> {
    let dir = themes_dir(app_data);
    std::fs::create_dir_all(&dir).map_err(AppError::Io)?;
    Ok(dir)
}

pub fn theme_dir(app_data: &Path, theme_id: &str) -> std::path::PathBuf {
    themes_dir(app_data).join(theme_id)
}

pub fn resolve_theme_dir(
    app_data: &Path,
    bundled_root: Option<&Path>,
    theme_id: &str,
) -> AppResult<std::path::PathBuf> {
    let theme_id = theme_id.trim();
    if theme_id.is_empty() || theme_id == "default" {
        return Err(AppError::user(
            "Cannot resolve directory for the default theme.",
        ));
    }

    let user_dir = theme_dir(app_data, theme_id);
    if user_dir.join("theme.yaml").is_file() {
        return Ok(user_dir);
    }

    if let Some(bundled_root) = bundled_root {
        let bundled_dir = bundled_root.join(theme_id);
        if bundled_dir.join("theme.yaml").is_file() {
            return Ok(bundled_dir);
        }
    }

    Err(AppError::user(format!(
        "Theme \"{theme_id}\" is not installed. Use Settings → Appearance → Install theme…"
    )))
}

fn load_theme_at_dir(dir: &Path, theme_id: &str) -> AppResult<LoadedTheme> {
    let manifest_path = dir.join("theme.yaml");
    if !manifest_path.is_file() {
        return Err(AppError::user(format!(
            "Theme \"{theme_id}\" is incomplete (missing theme.yaml). Reinstall the .svtheme package."
        )));
    }

    let manifest_raw = std::fs::read_to_string(&manifest_path).map_err(AppError::Io)?;
    let manifest = parse_manifest(&manifest_raw)?;
    if manifest.id != theme_id {
        return Err(AppError::user(format!(
            "Theme folder name ({theme_id}) does not match theme.yaml id ({})",
            manifest.id
        )));
    }

    let mut css_parts = Vec::new();
    for rel in &manifest.css {
        let path = dir.join(rel);
        if !path.is_file() {
            return Err(AppError::user(format!(
                "Theme \"{theme_id}\" is missing CSS file: {rel}. Reinstall the .svtheme package."
            )));
        }
        let raw = std::fs::read_to_string(&path).map_err(AppError::Io)?;
        css_parts.push(sanitize_css(&raw, rel)?);
    }

    let mut font_assets = Vec::new();
    for font in &manifest.fonts {
        let path = dir.join(&font.file);
        if !path.is_file() {
            return Err(AppError::user(format!(
                "Theme \"{theme_id}\" is missing font file: {}. Reinstall the .svtheme package.",
                font.file
            )));
        }
        let bytes = std::fs::read(&path).map_err(AppError::Io)?;
        validate_font_bytes(&bytes, &font.file)?;
        if !font.file.ends_with(".woff2") {
            return Err(AppError::user(format!(
                "Only .woff2 fonts are supported (found {}).",
                font.file
            )));
        }
        font_assets.push(ThemeFontAsset {
            family: font.family.clone(),
            weight: font.weight,
            relative_path: font.file.replace('\\', "/"),
        });
    }

    let font_faces = String::new();

    let mut layouts = ThemeLayoutConfig::default();
    for rel in &manifest.layouts {
        let path = dir.join(rel);
        if !path.is_file() {
            return Err(AppError::user(format!(
                "Theme \"{theme_id}\" is missing layout file: {rel}. Reinstall the .svtheme package."
            )));
        }
        let raw = std::fs::read_to_string(&path).map_err(AppError::Io)?;
        let partial: ThemeLayoutConfig = serde_json::from_str(&raw)
            .map_err(|e| AppError::user(format!("Invalid layout {rel}: {e}")))?;
        layouts.slots.extend(partial.slots);
    }

    Ok(LoadedTheme {
        summary: ThemeSummary {
            id: manifest.id.clone(),
            name: manifest.name.clone(),
            author: manifest.author.clone(),
            description: manifest.description.clone(),
        },
        css: ThemeCssBundle {
            id: manifest.id.clone(),
            css: css_parts.join("\n"),
            font_faces,
        },
        layouts,
        fonts: font_assets,
    })
}

pub fn parse_manifest(raw: &str) -> AppResult<ThemeManifest> {
    let manifest: ThemeManifest = serde_yaml::from_str(raw)
        .map_err(|e| AppError::user(format!("Invalid theme.yaml: {e}")))?;
    if manifest.id.trim().is_empty() {
        return Err(AppError::user("theme.yaml must include a non-empty id."));
    }
    if manifest.name.trim().is_empty() {
        return Err(AppError::user("theme.yaml must include a non-empty name."));
    }
    Ok(manifest)
}

pub fn sanitize_css(css: &str, theme_root_label: &str) -> AppResult<String> {
    if css.len() > MAX_CSS_BYTES {
        return Err(AppError::user(format!(
            "Theme CSS exceeds {MAX_CSS_BYTES} bytes in {theme_root_label}."
        )));
    }
    let lower = css.to_lowercase();
    let forbidden = [
        "@import",
        "javascript:",
        "expression(",
        "-moz-binding",
        "behavior:",
        "<script",
        "</script",
    ];
    for needle in forbidden {
        if lower.contains(needle) {
            return Err(AppError::user(format!(
                "Theme CSS in {theme_root_label} contains forbidden pattern: {needle}"
            )));
        }
    }
    for line in css.lines() {
        if let Some(idx) = line.to_lowercase().find("url(") {
            let rest = &line[idx + 4..];
            let trimmed = rest.trim_start();
            if trimmed.starts_with("http://")
                || trimmed.starts_with("https://")
                || trimmed.starts_with("//")
            {
                return Err(AppError::user(format!(
                    "Theme CSS in {theme_root_label} may only use relative asset URLs."
                )));
            }
        }
    }
    Ok(css.to_string())
}

pub fn validate_font_bytes(bytes: &[u8], path: &str) -> AppResult<()> {
    if bytes.len() > MAX_FONT_BYTES {
        return Err(AppError::user(format!(
            "Font file {path} exceeds {MAX_FONT_BYTES} bytes."
        )));
    }
    Ok(())
}

pub fn load_theme_from_dir(
    app_data: &Path,
    bundled_root: Option<&Path>,
    theme_id: &str,
) -> AppResult<LoadedTheme> {
    let theme_id = theme_id.trim();
    if theme_id.is_empty() || theme_id == "default" {
        return Ok(default_theme());
    }

    let dir = resolve_theme_dir(app_data, bundled_root, theme_id)?;
    load_theme_at_dir(&dir, theme_id)
}

fn theme_summary_from_dir(dir: &Path) -> Option<ThemeSummary> {
    let manifest_path = dir.join("theme.yaml");
    if !manifest_path.is_file() {
        return None;
    }
    let raw = std::fs::read_to_string(&manifest_path).ok()?;
    let manifest = parse_manifest(&raw).ok()?;
    let theme_id = dir.file_name()?.to_str()?;
    if manifest.id != theme_id {
        return None;
    }
    if load_theme_at_dir(dir, theme_id).is_err() {
        return None;
    }
    Some(ThemeSummary {
        id: manifest.id,
        name: manifest.name,
        author: manifest.author,
        description: manifest.description,
    })
}

fn collect_theme_summaries(root: &Path) -> AppResult<Vec<ThemeSummary>> {
    let mut themes = Vec::new();
    if !root.is_dir() {
        return Ok(themes);
    }
    for entry in std::fs::read_dir(root).map_err(AppError::Io)? {
        let entry = entry.map_err(AppError::Io)?;
        if !entry.file_type().map_err(AppError::Io)?.is_dir() {
            continue;
        }
        if let Some(summary) = theme_summary_from_dir(&entry.path()) {
            themes.push(summary);
        }
    }
    Ok(themes)
}

pub fn list_installed_themes(
    app_data: &Path,
    bundled_root: Option<&Path>,
) -> AppResult<Vec<ThemeSummary>> {
    let mut by_id = std::collections::HashMap::new();
    by_id.insert(
        "default".to_string(),
        ThemeSummary {
            id: "default".into(),
            name: "Default".into(),
            author: Some("Supervisor".into()),
            description: Some("Built-in dark theme".into()),
        },
    );

    if let Some(bundled_root) = bundled_root {
        for summary in collect_theme_summaries(bundled_root)? {
            by_id.entry(summary.id.clone()).or_insert(summary);
        }
    }

    for summary in collect_theme_summaries(&themes_dir(app_data))? {
        by_id.insert(summary.id.clone(), summary);
    }

    let mut themes: Vec<ThemeSummary> = by_id.into_values().collect();
    themes.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(themes)
}

pub fn default_theme() -> LoadedTheme {
    LoadedTheme {
        summary: ThemeSummary {
            id: "default".into(),
            name: "Default".into(),
            author: Some("Supervisor".into()),
            description: Some("Built-in dark theme".into()),
        },
        css: ThemeCssBundle {
            id: "default".into(),
            css: String::new(),
            font_faces: String::new(),
        },
        layouts: ThemeLayoutConfig::default(),
        fonts: Vec::new(),
    }
}

pub fn install_theme_archive(app_data: &Path, archive_path: &Path) -> AppResult<ThemeSummary> {
    ensure_themes_dir(app_data)?;
    let file = std::fs::File::open(archive_path).map_err(AppError::Io)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::user(format!("Invalid theme archive: {e}")))?;

    let entry_names: Vec<String> = (0..archive.len())
        .map(|i| {
            archive
                .by_index(i)
                .map(|e| e.name().to_string())
                .map_err(|e| AppError::user(e.to_string()))
        })
        .collect::<Result<_, _>>()?;
    let strip_prefix = archive_strip_prefix(&entry_names);

    let mut manifest: Option<ThemeManifest> = None;
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| AppError::user(e.to_string()))?;
        let rel = relative_archive_path(entry.name(), strip_prefix.as_deref());
        if rel.ends_with('/') {
            continue;
        }
        if rel == "theme.yaml" || rel.ends_with("/theme.yaml") {
            let mut raw = String::new();
            std::io::Read::read_to_string(&mut entry, &mut raw).map_err(AppError::Io)?;
            manifest = Some(parse_manifest(&raw)?);
            break;
        }
    }
    let manifest =
        manifest.ok_or_else(|| AppError::user("Theme archive must contain theme.yaml."))?;

    let dest = theme_dir(app_data, &manifest.id);
    if dest.exists() {
        std::fs::remove_dir_all(&dest).map_err(AppError::Io)?;
    }
    std::fs::create_dir_all(&dest).map_err(AppError::Io)?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| AppError::user(e.to_string()))?;
        let rel = relative_archive_path(entry.name(), strip_prefix.as_deref());
        if rel.ends_with('/') {
            continue;
        }
        if rel.contains("..") {
            return Err(AppError::user("Theme archive contains invalid paths."));
        }
        let lower = rel.to_lowercase();
        if lower.ends_with(".js")
            || lower.ends_with(".html")
            || lower.ends_with(".exe")
            || lower.ends_with(".bat")
            || lower.ends_with(".cmd")
        {
            return Err(AppError::user(format!(
                "Theme archive contains forbidden file type: {rel}"
            )));
        }
        let out_path = dest.join(&rel);
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(AppError::Io)?;
        }
        if lower.ends_with(".css") {
            let mut raw = String::new();
            std::io::Read::read_to_string(&mut entry, &mut raw).map_err(AppError::Io)?;
            let safe = sanitize_css(&raw, &rel)?;
            std::fs::write(&out_path, safe).map_err(AppError::Io)?;
        } else if lower.ends_with(".woff2") {
            let mut bytes = Vec::new();
            std::io::Read::read_to_end(&mut entry, &mut bytes).map_err(AppError::Io)?;
            validate_font_bytes(&bytes, &rel)?;
            std::fs::write(&out_path, bytes).map_err(AppError::Io)?;
        } else {
            let mut bytes = Vec::new();
            std::io::Read::read_to_end(&mut entry, &mut bytes).map_err(AppError::Io)?;
            std::fs::write(&out_path, bytes).map_err(AppError::Io)?;
        }
    }

    load_theme_from_dir(app_data, None, &manifest.id)?;
    Ok(ThemeSummary {
        id: manifest.id,
        name: manifest.name,
        author: manifest.author,
        description: manifest.description,
    })
}

fn normalize_zip_path(name: &str) -> String {
    name.replace('\\', "/")
}

fn relative_archive_path(name: &str, strip_prefix: Option<&str>) -> String {
    let mut path = normalize_zip_path(name);
    if path.ends_with('/') {
        return path;
    }
    path = path.trim_start_matches("./").to_string();
    if let Some(prefix) = strip_prefix {
        if let Some(stripped) = path.strip_prefix(prefix) {
            path = stripped.to_string();
        }
    }
    path
}

/// Strip a single wrapper folder (e.g. `my-theme/theme.yaml`) but keep real subfolders
/// like `layouts/shell.json` when `theme.yaml` lives at the archive root.
fn archive_strip_prefix(entry_names: &[String]) -> Option<String> {
    let files: Vec<String> = entry_names
        .iter()
        .map(|n| normalize_zip_path(n))
        .filter(|n| !n.ends_with('/'))
        .map(|n| n.trim_start_matches("./").to_string())
        .collect();

    if files.is_empty() {
        return None;
    }

    if files.iter().any(|n| n == "theme.yaml") {
        return None;
    }

    let mut roots = std::collections::HashSet::new();
    for path in &files {
        match path.split_once('/') {
            Some((root, _)) => {
                roots.insert(root.to_string());
            }
            None => return None,
        }
    }

    if roots.len() == 1 {
        Some(format!("{}/", roots.into_iter().next()?))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_external_urls_in_css() {
        let css = "body { background: url(https://evil.com/x.png); }";
        assert!(sanitize_css(css, "test.css").is_err());
    }

    #[test]
    fn rejects_imports() {
        let css = "@import url('x.css'); body {}";
        assert!(sanitize_css(css, "test.css").is_err());
    }

    #[test]
    fn allows_relative_urls() {
        let css = "body { background: url('assets/bg.png'); }";
        assert!(sanitize_css(css, "test.css").is_ok());
    }

    #[test]
    fn keeps_layout_subfolders_when_theme_yaml_at_archive_root() {
        let entries = vec![
            "theme.yaml".into(),
            "tokens.css".into(),
            "layouts/shell.json".into(),
        ];
        assert_eq!(archive_strip_prefix(&entries), None);
        assert_eq!(
            relative_archive_path("layouts/shell.json", None),
            "layouts/shell.json"
        );
    }

    #[test]
    fn strips_single_wrapper_folder_in_archive() {
        let entries = vec![
            "example/theme.yaml".into(),
            "example/tokens.css".into(),
            "example/layouts/shell.json".into(),
        ];
        assert_eq!(archive_strip_prefix(&entries), Some("example/".into()));
        assert_eq!(
            relative_archive_path("example/layouts/shell.json", Some("example/")),
            "layouts/shell.json"
        );
    }

    #[test]
    fn bundled_grid_and_windows_95_are_loadable() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../themes/bundled");
        for id in ["grid", "windows-95"] {
            let dir = root.join(id);
            let loaded = load_theme_at_dir(&dir, id).unwrap_or_else(|e| panic!("{id}: {e}"));
            assert_eq!(loaded.summary.id, id);
        }
        let summaries = collect_theme_summaries(&root).expect("bundled dir");
        let ids: Vec<_> = summaries.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&"grid"), "expected grid in {ids:?}");
        assert!(ids.contains(&"windows-95"), "expected windows-95 in {ids:?}");
    }
}
