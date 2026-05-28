/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
//! Native About dialog via the Tauri dialog plugin (system message box on Windows).

use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

pub fn show(app: &tauri::AppHandle) -> Result<(), String> {
    let pkg = app.package_info();
    let body = format!(
        "Version {}\n\nCross-game mod manager and launcher for Windows.",
        pkg.version
    );
    app.dialog()
        .message(body)
        .title(format!("About {}", pkg.name))
        .kind(MessageDialogKind::Info)
        .blocking_show();
    Ok(())
}
