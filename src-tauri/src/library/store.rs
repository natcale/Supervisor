/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use super::{now_ts, GameLibrary};
use crate::errors::{AppError, AppResult};
use std::fs;
use std::path::{Path, PathBuf};

fn library_path(app_data: &Path, game_id: &str) -> PathBuf {
    app_data.join("library").join(game_id).join("library.json")
}

pub fn load_library(app_data: &Path, game_id: &str) -> AppResult<GameLibrary> {
    let path = library_path(app_data, game_id);
    if !path.is_file() {
        return Ok(GameLibrary {
            game_id: game_id.to_string(),
            mods: Vec::new(),
            updated_at: now_ts(),
        });
    }
    let raw = fs::read_to_string(&path).map_err(AppError::Io)?;
    serde_json::from_str(&raw).map_err(|e| AppError::user(format!("Corrupt library file: {e}")))
}

pub fn save_library(app_data: &Path, library: &GameLibrary) -> AppResult<()> {
    let path = library_path(app_data, &library.game_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(AppError::Io)?;
    }
    let raw = serde_json::to_string_pretty(library).map_err(|e| AppError::user(e.to_string()))?;
    fs::write(path, raw).map_err(AppError::Io)
}
