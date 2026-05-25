/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::errors::{AppError, AppResult};
use crate::install::{apply_fomod_selection, apply_install_chain, has_fomod, parse_fomod_config};
use crate::library::{InstallState, LibraryMod, NexusMeta, now_ts};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestedMod {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub files: Vec<String>,
    pub dependencies: Vec<String>,
    pub install_state: InstallState,
    pub needs_fomod: bool,
    pub nexus: Option<NexusMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestResult {
    pub mods: Vec<IngestedMod>,
    pub staging_dir: String,
}

pub fn ingest_paths(staging_root: &Path, paths: &[String]) -> AppResult<IngestResult> {
    fs::create_dir_all(staging_root).map_err(AppError::Io)?;

    let mut mods = Vec::new();

    for raw in paths {
        let source = PathBuf::from(raw);
        if !source.exists() {
            return Err(AppError::user(format!(
                "Could not find \"{}\" — it may have been moved or deleted.",
                source.display()
            )));
        }

        let slug = slug_from_path(&source);
        let mod_root = staging_root.join(&slug);
        if mod_root.exists() {
            fs::remove_dir_all(&mod_root).map_err(AppError::Io)?;
        }
        fs::create_dir_all(&mod_root).map_err(AppError::Io)?;

        if source.is_dir() {
            copy_dir_recursive(&source, &mod_root)?;
        } else if is_zip(&source) {
            extract_zip(&source, &mod_root)?;
        } else if is_seven_zip(&source) {
            extract_7z(&source, &mod_root)?;
        } else {
            let file_name = source
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("mod.bin");
            fs::copy(&source, mod_root.join(file_name)).map_err(AppError::Io)?;
        }

        let needs_fomod = apply_install_chain(&mod_root)?;
        let install_state = if needs_fomod {
            InstallState::PendingFomod
        } else {
            InstallState::Installed
        };

        let files = if needs_fomod {
            Vec::new()
        } else {
            list_files_relative(&mod_root, staging_root)?
        };

        if !needs_fomod && files.is_empty() {
            continue;
        }

        let dependencies = parse_dependencies(&mod_root);

        mods.push(IngestedMod {
            id: format!("local-{}", Uuid::new_v4()),
            name: slug.replace('-', " "),
            slug,
            files,
            dependencies,
            install_state,
            needs_fomod,
            nexus: None,
        });
    }

    Ok(IngestResult {
        mods,
        staging_dir: staging_root.to_string_lossy().into_owned(),
    })
}

pub fn finalize_fomod_mod(
    staging_root: &Path,
    mod_id: &str,
    slug: &str,
    selections: &[String],
) -> AppResult<IngestedMod> {
    let mod_root = staging_root.join(slug);
    if !has_fomod(&mod_root) {
        return Err(AppError::user("This mod does not have a FOMOD installer"));
    }
    apply_fomod_selection(&mod_root, selections)?;
    let files = list_files_relative(&mod_root, staging_root)?;
    Ok(IngestedMod {
        id: mod_id.to_string(),
        name: slug.replace('-', " "),
        slug: slug.to_string(),
        files,
        dependencies: Vec::new(),
        install_state: InstallState::Installed,
        needs_fomod: false,
        nexus: None,
    })
}

pub fn parse_fomod_for_slug(staging_root: &Path, slug: &str) -> AppResult<crate::install::FomodConfig> {
    let mod_root = staging_root.join(slug);
    let config_path = crate::install::find_module_config(&mod_root)
        .ok_or_else(|| AppError::user("FOMOD config not found"))?;
    let xml = fs::read_to_string(config_path).map_err(AppError::Io)?;
    Ok(parse_fomod_config(&xml))
}

pub fn refresh_mod_from_staging(
    staging_root: &Path,
    mod_id: &str,
    slug: &str,
    name: &str,
    nexus: Option<NexusMeta>,
) -> AppResult<IngestedMod> {
    let mod_root = staging_root.join(slug);
    if !mod_root.is_dir() {
        return Err(AppError::user(format!(
            "Mod folder not found at \"{}\".",
            mod_root.display()
        )));
    }

    let needs_fomod = has_fomod(&mod_root);
    let install_state = if needs_fomod {
        InstallState::PendingFomod
    } else {
        InstallState::Installed
    };
    let files = if needs_fomod {
        Vec::new()
    } else {
        list_files_relative(&mod_root, staging_root)?
    };

    Ok(IngestedMod {
        id: mod_id.to_string(),
        name: name.to_string(),
        slug: slug.to_string(),
        files,
        dependencies: Vec::new(),
        install_state,
        needs_fomod,
        nexus,
    })
}

pub fn ingested_to_library(entry: &IngestedMod) -> LibraryMod {
    LibraryMod {
        id: entry.id.clone(),
        name: entry.name.clone(),
        slug: entry.slug.clone(),
        files: entry.files.clone(),
        dependencies: entry.dependencies.clone(),
        install_state: entry.install_state.clone(),
        installed_at: now_ts(),
        nexus: entry.nexus.clone(),
        notes: None,
    }
}

fn slug_from_path(path: &Path) -> String {
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("mod");
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .to_lowercase()
}

fn is_zip(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("zip"))
        .unwrap_or(false)
}

fn is_seven_zip(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            let lower = e.to_lowercase();
            lower == "7z" || lower == "rar"
        })
        .unwrap_or(false)
}

fn parse_dependencies(mod_root: &Path) -> Vec<String> {
    let meta = mod_root.join("meta.ini");
    if !meta.is_file() {
        return Vec::new();
    }
    let Ok(content) = fs::read_to_string(&meta) else {
        return Vec::new();
    };
    let mut in_deps = false;
    let mut deps = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_deps = trimmed.eq_ignore_ascii_case("[Dependencies]");
            continue;
        }
        if in_deps && !trimmed.is_empty() && !trimmed.starts_with(';') {
            if let Some((_, value)) = trimmed.split_once('=') {
                let v = value.trim();
                if !v.is_empty() {
                    deps.push(v.to_string());
                }
            } else {
                deps.push(trimmed.to_string());
            }
        }
    }
    deps
}

fn extract_7z(archive: &Path, dest: &Path) -> AppResult<()> {
    sevenz_rust::decompress_file(archive, dest).map_err(|e| {
        AppError::user(format!(
            "Could not extract {}: {e}. Install 7-Zip or use a .zip archive.",
            archive.display()
        ))
    })
}

fn copy_dir_recursive(from: &Path, to: &Path) -> AppResult<()> {
    fs::create_dir_all(to).map_err(AppError::Io)?;
    for entry in fs::read_dir(from).map_err(AppError::Io)? {
        let entry = entry.map_err(AppError::Io)?;
        let ty = entry.file_type().map_err(AppError::Io)?;
        let dest = to.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &dest)?;
        } else {
            fs::copy(entry.path(), dest).map_err(AppError::Io)?;
        }
    }
    Ok(())
}

fn extract_zip(archive: &Path, dest: &Path) -> AppResult<()> {
    let file = fs::File::open(archive).map_err(AppError::Io)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::user(format!("Could not read archive: {e}")))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| AppError::user(format!("Could not read archive entry: {e}")))?;
        let outpath = match entry.enclosed_name() {
            Some(path) => dest.join(path),
            None => continue,
        };
        if entry.name().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(AppError::Io)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent).map_err(AppError::Io)?;
            }
            let mut outfile = fs::File::create(&outpath).map_err(AppError::Io)?;
            std::io::copy(&mut entry, &mut outfile).map_err(AppError::Io)?;
        }
    }
    Ok(())
}

fn list_files_relative(mod_root: &Path, staging_root: &Path) -> AppResult<Vec<String>> {
    let mut files = Vec::new();
    walk(mod_root, staging_root, &mut files)?;
    files.sort();
    Ok(files)
}

fn walk(current: &Path, staging_root: &Path, out: &mut Vec<String>) -> AppResult<()> {
    for entry in fs::read_dir(current).map_err(AppError::Io)? {
        let entry = entry.map_err(AppError::Io)?;
        let path = entry.path();
        if path.is_dir() {
            walk(&path, staging_root, out)?;
        } else if path.is_file() {
            let rel = path
                .strip_prefix(staging_root)
                .map_err(|_| AppError::user("Unexpected staging path"))?;
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}
