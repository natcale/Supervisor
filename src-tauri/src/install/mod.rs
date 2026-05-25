/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
mod fomod;
mod installers;

pub use fomod::{apply_fomod_selection, has_fomod, parse_fomod_config, FomodConfig};
pub use installers::{apply_install_chain, find_module_config};
use crate::games::{GameProfile, MergeMode, ModTypeDef};
use crate::root_builder::classify_root_files;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct NormalizedFile {
    pub source: String,
    pub deploy_rel: String,
    pub is_root: bool,
}

impl NormalizedFile {
    pub fn deploy_key(&self) -> String {
        if self.is_root {
            format!("root:{}", self.deploy_rel.replace('\\', "/").to_lowercase())
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
    let inner = strip_known_wrappers(&stripped, &mod_type);

    let mod_type_def = profile.mod_type(&mod_type).unwrap_or_else(|| profile.default_mod_type());

    let staging_paths: Vec<String> = inner
        .iter()
        .map(|(rel, _)| format!("{slug}/{rel}"))
        .collect();
    let (root_entries, _) = classify_root_files(staging, &staging_paths);
    let root_sources: std::collections::HashSet<String> =
        root_entries.iter().map(|r| r.source.clone()).collect();

    let mut normalized_files = Vec::new();
    for (rel, _original) in inner {
        let source = format!("{slug}/{rel}");
        let is_root = root_sources.contains(&source);
        let deploy_rel = compute_deploy_rel(profile, &mod_type_def, slug, &rel);

        normalized_files.push(NormalizedFile {
            source,
            deploy_rel,
            is_root,
        });
    }

    NormalizedMod {
        mod_id: mod_id.to_string(),
        mod_type,
        files: normalized_files,
    }
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

fn strip_known_wrappers(paths: &[String], mod_type: &str) -> Vec<(String, String)> {
    let wrappers = match mod_type {
        "default" | "dinput" | "bg3_loose" | "root" => &["Data/", "data/"][..],
        "cp77_legacy" => &["archive/pc/mod/", "archive/pc/mod"][..],
        "cp77_redmod" => &["mods/"][..],
        "bg3_pak" => &["Mods/"][..],
        "bepinex" => &["BepInEx/plugins/", "BepInEx/"][..],
        "qmod" => &["QMods/"][..],
        "smf" => &["Simple Mod Framework/Mods/"][..],
        "pak" => &["MarvelGame/Marvel/Content/Paks/~mods/"][..],
        _ => &[][..],
    };

    let mut stripped = paths.to_vec();
    for wrapper in wrappers {
        if stripped.iter().all(|p| p.starts_with(wrapper) || p == wrapper.trim_end_matches('/')) {
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

    stripped
        .into_iter()
        .map(|p| (p.clone(), p))
        .collect()
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
                p.starts_with("archives/")
                    || p.starts_with("scripts/")
                    || p.starts_with("tweaks/")
            })
        {
            return "cp77_redmod".into();
        }
        if stripped.iter().any(|p| p.ends_with(".archive")) {
            return "cp77_legacy".into();
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
        if stripped.iter().any(|p| p.starts_with("Data/") || p.starts_with("data/")) {
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

    if mod_root.join("BepInEx").is_dir()
        || stripped.iter().any(|p| p.starts_with("BepInEx/"))
        || has_dll_under(&mod_root, "BepInEx")
    {
        if profile.mod_type("bepinex").is_some() {
            return "bepinex".into();
        }
    }

    if mod_root.join("mod.json").is_file()
        || stripped.iter().any(|p| p.starts_with("QMods/") || p.ends_with("mod.json"))
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

fn has_dll_under(root: &Path, sub: &str) -> bool {
    let dir = root.join(sub);
    if !dir.is_dir() {
        return false;
    }
    walk_has_dll(&dir)
}

fn walk_has_dll(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if walk_has_dll(&path) {
                return true;
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("dll") {
            return true;
        }
    }
    false
}
