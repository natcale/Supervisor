/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
mod fomod;
mod installers;

use crate::games::{GameProfile, MergeMode, ModTypeDef};
use crate::root_builder::classify_root_files;
pub use fomod::{apply_fomod_selection, has_fomod, parse_fomod_config, FomodConfig};
pub use installers::{apply_install_chain, find_module_config};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct NormalizedFile {
    pub source: String,
    pub deploy_rel: String,
    /// True when the file deploys to the game install root by basename (SKSE, doorstop).
    pub is_root: bool,
    pub mod_type: String,
}

impl NormalizedFile {
    pub fn deploy_key(&self) -> String {
        if self.is_root {
            format!(
                "root:{}",
                Path::new(&self.deploy_rel)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&self.deploy_rel)
                    .to_lowercase()
            )
        } else {
            self.deploy_rel.replace('\\', "/").to_lowercase()
        }
    }
}

#[derive(Debug, Clone)]
pub struct NormalizedMod {
    pub mod_id: String,
    pub mod_type: String,
    pub files: Vec<NormalizedFile>,
}

pub fn normalize_mod(
    staging: &Path,
    mod_id: &str,
    slug: &str,
    files: &[String],
    profile: &GameProfile,
    _deploy_path_override: Option<&str>,
) -> NormalizedMod {
    let stripped = strip_slug_prefix(files, slug);
    let mod_type = detect_mod_type(staging, slug, &stripped, profile);
    let mod_root = staging.join(slug);
    let inner = strip_known_wrappers(&stripped, &mod_type, profile);

    let mut normalized_files = Vec::new();
    for (rel, _original) in inner {
        let source = format!("{slug}/{rel}");
        let file_type = route_file_mod_type(profile, &mod_type, &mod_root, &rel);
        let mod_type_def = profile
            .mod_type(&file_type)
            .unwrap_or_else(|| profile.default_mod_type());
        let deploy_rel = compute_deploy_rel(profile, mod_type_def, slug, &rel);
        let is_root = is_game_root_basename_deploy(&file_type);

        normalized_files.push(NormalizedFile {
            source,
            deploy_rel,
            is_root,
            mod_type: file_type,
        });
    }

    NormalizedMod {
        mod_id: mod_id.to_string(),
        mod_type,
        files: normalized_files,
    }
}

fn is_game_root_basename_deploy(mod_type: &str) -> bool {
    matches!(mod_type, "dinput" | "doorstop" | "bg3_se" | "root")
}

fn compute_deploy_rel(
    profile: &GameProfile,
    mod_type: &ModTypeDef,
    slug: &str,
    rel: &str,
) -> String {
    if mod_type.id == "cp77_redmod" {
        if rel.is_empty() {
            return slug.to_string();
        }
        return format!("{slug}/{rel}");
    }

    if profile.merge_mode == MergeMode::PerModFolder {
        let folder = slug;
        if rel.is_empty() {
            return folder.to_string();
        }
        return format!("{folder}/{rel}");
    }

    if mod_type.id == "default" && mod_type.rel_path.eq_ignore_ascii_case("data") {
        return rel.to_string();
    }

    if mod_type.rel_path == "." {
        return rel.to_string();
    }

    rel.to_string()
}

fn strip_slug_prefix(files: &[String], slug: &str) -> Vec<String> {
    let prefix = format!("{slug}/");
    files
        .iter()
        .map(|f| {
            if f.starts_with(&prefix) {
                f[prefix.len()..].to_string()
            } else {
                f.clone()
            }
        })
        .collect()
}

fn strip_known_wrappers(
    paths: &[String],
    mod_type: &str,
    profile: &GameProfile,
) -> Vec<(String, String)> {
    let static_wrappers: &[&str] = match mod_type {
        "default" | "dinput" | "bg3_loose" | "root" => &["Data/", "data/"],
        "cp77_legacy" => &["archive/pc/mod/", "archive/pc/mod"],
        "cp77_redmod" => &["mods/"],
        "bg3_pak" => &["Mods/"],
        "bepinex" => &["BepInEx/plugins/", "BepInEx/"],
        "bepinex_tree" | "doorstop" => &[],
        "qmod" => &["QMods/"],
        "smf" => &["Simple Mod Framework/Mods/"],
        "win64" => &["Binaries/Win64/"],
        "pak" => &["Content/Paks/~mods/"],
        _ => &[],
    };

    let mut wrappers: Vec<String> = static_wrappers.iter().map(|w| (*w).to_string()).collect();
    if mod_type == "pak" {
        if let Some(pak) = profile.mod_type("pak") {
            wrappers.push(format!("{}/", pak.rel_path));
        }
    }
    if mod_type == "win64" {
        if let Some(win64) = profile.mod_type("win64") {
            wrappers.push(format!("{}/", win64.rel_path));
        }
    }

    let mut stripped = paths.to_vec();
    for wrapper in &wrappers {
        if stripped
            .iter()
            .all(|p| p.starts_with(wrapper) || p == wrapper.trim_end_matches('/'))
        {
            stripped = stripped
                .iter()
                .map(|p| {
                    p.strip_prefix(wrapper)
                        .unwrap_or(p)
                        .trim_start_matches('/')
                        .to_string()
                })
                .collect();
        }
    }

    stripped.into_iter().map(|p| (p.clone(), p)).collect()
}

fn route_file_mod_type(
    profile: &GameProfile,
    primary: &str,
    _mod_root: &Path,
    rel: &str,
) -> String {
    let lower = rel.to_lowercase();

    if profile.id == "marvelrivals" {
        if marvel_rivals_pak_asset(rel) {
            return "pak".into();
        }
        if marvel_rivals_win64_asset(rel) {
            return "win64".into();
        }
        return primary.into();
    }

    if profile.mod_type("bepinex_tree").is_some() && lower.starts_with("bepinex/") {
        return "bepinex_tree".into();
    }

    if profile.mod_type("doorstop").is_some() && is_doorstop_loader(rel) {
        return "doorstop".into();
    }

    if profile.mod_type("dinput").is_some() && is_script_extender_loader(rel) {
        return "dinput".into();
    }

    if profile.mod_type("bepinex").is_some()
        && (lower.starts_with("plugins/") || lower.contains("/plugins/"))
    {
        return "bepinex".into();
    }

    primary.into()
}

fn detect_mod_type(
    staging: &Path,
    slug: &str,
    stripped: &[String],
    profile: &GameProfile,
) -> String {
    let mod_root = staging.join(slug);

    if profile.id == "cyberpunk2077" {
        if mod_root.join("info.json").is_file()
            || stripped.iter().any(|p| {
                p.starts_with("archives/") || p.starts_with("scripts/") || p.starts_with("tweaks/")
            })
        {
            return "cp77_redmod".into();
        }
        if stripped.iter().any(|p| p.ends_with(".archive")) {
            return "cp77_legacy".into();
        }
    }

    if profile.id == "marvelrivals" {
        let has_pak_assets = stripped.iter().any(|p| marvel_rivals_pak_asset(p));
        let has_win64_assets = stripped.iter().any(|p| marvel_rivals_win64_asset(p))
            || mod_root.join("dsound.dll").is_file()
            || mod_root.join("version.dll").is_file()
            || mod_root.join("winmm.dll").is_file();
        if has_win64_assets && !has_pak_assets {
            return "win64".into();
        }
        if has_pak_assets {
            return "pak".into();
        }
    }

    if profile.id == "baldursgate3" {
        if stripped.iter().any(|p| p.ends_with(".pak")) {
            return "bg3_pak".into();
        }
        let staging_paths: Vec<String> = stripped.iter().map(|p| format!("{slug}/{p}")).collect();
        let (root_entries, _) = classify_root_files(staging, &staging_paths);
        if !root_entries.is_empty() {
            return "bg3_se".into();
        }
        if stripped
            .iter()
            .any(|p| p.starts_with("Data/") || p.starts_with("data/"))
        {
            return "bg3_loose".into();
        }
    }

    if mod_root.join("manifest.json").is_file() {
        if profile.mod_type("smf").is_some() {
            return "smf".into();
        }
    }

    if crate::install::installers::find_module_config(&mod_root).is_some() {
        return profile.default_mod_type().id.clone();
    }

    if profile.mod_type("bepinex_tree").is_some() || profile.mod_type("bepinex").is_some() {
        if mod_root.join("BepInEx").is_dir() || stripped.iter().any(|p| p.starts_with("BepInEx/")) {
            if is_bepinex_framework(&mod_root, stripped) {
                return "bepinex_tree".into();
            }
            if profile.mod_type("bepinex").is_some() {
                return "bepinex".into();
            }
        }
        if stripped.iter().any(|p| is_doorstop_loader(p)) {
            return "doorstop".into();
        }
    }

    if mod_root.join("mod.json").is_file()
        || stripped
            .iter()
            .any(|p| p.starts_with("QMods/") || p.ends_with("mod.json"))
    {
        if profile.mod_type("qmod").is_some() {
            return "qmod".into();
        }
    }

    if stripped.iter().any(|p| p.ends_with(".pak")) && profile.mod_type("pak").is_some() {
        return "pak".into();
    }

    if profile.mod_type("dinput").is_some() {
        let staging_paths: Vec<String> = stripped.iter().map(|p| format!("{slug}/{p}")).collect();
        let (root_entries, data_entries) = classify_root_files(staging, &staging_paths);
        if !root_entries.is_empty() && root_entries.len() >= data_entries.len() {
            return "dinput".into();
        }
    }

    profile.default_mod_type().id.clone()
}

fn marvel_rivals_pak_asset(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.ends_with(".pak") || lower.ends_with(".ucas") || lower.ends_with(".utoc")
}

fn marvel_rivals_win64_asset(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.contains("binaries/win64") || lower.ends_with(".dll") || lower.ends_with(".asi")
}

fn is_doorstop_loader(path: &str) -> bool {
    let name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
        .to_lowercase();
    matches!(
        name.as_str(),
        "winhttp.dll" | "version.dll" | "doorstop_config.ini" | ".doorstop_version"
    )
}

fn is_script_extender_loader(path: &str) -> bool {
    let lower = path.to_lowercase();
    let name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
        .to_lowercase();

    let staging_paths = [path.to_string()];
    let (root_entries, _) = classify_root_files(Path::new(""), &staging_paths);
    if root_entries.is_empty() {
        return false;
    }

    !lower.starts_with("bepinex/")
        && !lower.starts_with("data/")
        && !lower.contains("binaries/win64")
        && (name.ends_with(".dll")
            || name.ends_with(".exe")
            || name.contains("loader")
            || name.contains("scriptextender"))
}

fn is_bepinex_framework(mod_root: &Path, stripped: &[String]) -> bool {
    mod_root.join("BepInEx/core").is_dir()
        || mod_root.join("BepInEx/patchers").is_dir()
        || stripped.iter().any(|p| {
            let l = p.to_lowercase();
            l.starts_with("bepinex/core/")
                || l.starts_with("bepinex/patchers/")
                || l.starts_with("bepinex/cache/")
                || l == "bepinex"
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::profile_by_id;
    use std::fs;

    fn test_staging() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("supervisor-install-test-{}", uuid::Uuid::new_v4()))
    }

    #[test]
    fn marvel_utoc_bypass_routes_dlls_to_win64_not_game_root() {
        let profile = profile_by_id("marvelrivals").expect("marvelrivals");
        let staging = test_staging();
        let slug = "utoc-bypass";
        let mod_root = staging.join(slug);
        fs::create_dir_all(&mod_root).unwrap();
        fs::write(mod_root.join("dsound.dll"), b"x").unwrap();
        fs::write(mod_root.join("version.dll"), b"x").unwrap();

        let files = vec![format!("{slug}/dsound.dll"), format!("{slug}/version.dll")];
        let normalized = normalize_mod(&staging, "mod-1", slug, &files, profile, None);

        assert_eq!(normalized.mod_type, "win64");
        for file in &normalized.files {
            assert_eq!(file.mod_type, "win64");
            assert!(
                !file.is_root,
                "{} should not deploy to game root",
                file.deploy_rel
            );
        }
        let _ = fs::remove_dir_all(&staging);
    }

    #[test]
    fn marvel_mixed_mod_splits_pak_and_win64_per_file() {
        let profile = profile_by_id("marvelrivals").expect("marvelrivals");
        let staging = test_staging();
        let slug = "mixed-mod";
        let files = vec![
            format!("{slug}/skin.pak"),
            format!("{slug}/skin.utoc"),
            format!("{slug}/skin.ucas"),
            format!("{slug}/dsound.dll"),
        ];
        let normalized = normalize_mod(&staging, "mod-2", slug, &files, profile, None);

        let types: Vec<_> = normalized
            .files
            .iter()
            .map(|f| f.mod_type.as_str())
            .collect();
        assert!(types.contains(&"pak"));
        assert!(types.contains(&"win64"));
        let _ = fs::remove_dir_all(&staging);
    }

    #[test]
    fn valheim_bepinex_framework_routes_tree_and_doorstop() {
        let profile = profile_by_id("valheim").expect("valheim");
        let staging = test_staging();
        let slug = "bepinex-pack";
        let mod_root = staging.join(slug);
        fs::create_dir_all(mod_root.join("BepInEx/core")).unwrap();
        fs::write(mod_root.join("winhttp.dll"), b"x").unwrap();
        fs::write(mod_root.join("BepInEx/core/BepInEx.dll"), b"x").unwrap();

        let files = vec![
            format!("{slug}/BepInEx/core/BepInEx.dll"),
            format!("{slug}/winhttp.dll"),
        ];
        let normalized = normalize_mod(&staging, "mod-3", slug, &files, profile, None);

        assert_eq!(normalized.mod_type, "bepinex_tree");
        let winhttp = normalized
            .files
            .iter()
            .find(|f| f.deploy_rel.contains("winhttp.dll"))
            .expect("winhttp");
        assert_eq!(winhttp.mod_type, "doorstop");
        assert!(winhttp.is_root);
        let _ = fs::remove_dir_all(&staging);
    }
}
