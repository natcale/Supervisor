/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::errors::{AppError, UserFacingIssue};

#[tauri::command]
pub fn show_about_dialog(app: tauri::AppHandle) -> Result<(), UserFacingIssue> {
    crate::about::show(&app).map_err(|e| AppError::user(e).to_user_issue())
}
