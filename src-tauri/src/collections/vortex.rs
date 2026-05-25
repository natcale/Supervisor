/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

use crate::errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionModEntry {
    pub name: String,
    pub version: Option<String>,
    pub domain_name: Option<String>,
    pub mod_id: Option<u64>,
    pub file_id: Option<u64>,
    pub optional: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionManifest {
    pub name: Option<String>,
    pub game: Option<String>,
    #[serde(default)]
    pub mods: Vec<CollectionModEntry>,
}

#[derive(Debug, Clone)]
pub struct ParsedCollection {
    pub name: String,
    pub game_hint: Option<String>,
    pub mods: Vec<CollectionModEntry>,
}

pub fn parse_collection_file(path: &Path) -> AppResult<ParsedCollection> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext == "collection" || ext == "zip" {
        return parse_collection_archive(path);
    }

    Err(AppError::user(
        "Unsupported collection format. Use a Vortex .collection or .zip bundle.",
    ))
}

fn parse_collection_archive(path: &Path) -> AppResult<ParsedCollection> {
    let file = File::open(path).map_err(AppError::Io)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::user(format!("Could not read collection archive: {e}")))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| AppError::user(format!("Bad collection entry: {e}")))?;
        let name = entry.name().to_lowercase();
        if name.ends_with("collection.json") || name.ends_with("manifest.json") {
            let mut raw = String::new();
            entry.read_to_string(&mut raw).map_err(AppError::Io)?;
            let manifest: CollectionManifest =
                serde_json::from_str(&raw).map_err(|e| AppError::user(format!("Bad manifest: {e}")))?;
            return Ok(ParsedCollection {
                name: manifest
                    .name
                    .unwrap_or_else(|| "Imported Collection".into()),
                game_hint: manifest.game,
                mods: manifest.mods,
            });
        }
    }

    Err(AppError::user(
        "Collection archive does not contain collection.json or manifest.json.",
    ))
}
