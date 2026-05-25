/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::hardlink::RootFileEntry;
use std::path::Path;

const ROOT_EXTENSIONS: &[&str] = &[
    "dll", "exe", "asi", "dx11", "dx12", "nvapi", "winmm", "d3d11", "d3d12",
];

const ROOT_NAMES: &[&str] = &[
    "skse64_loader.exe",
    "skse_loader.exe",
    "f4se_loader.exe",
    "sfse_loader.exe",
    "nvse_loader.exe",
    "dinput8.dll",
    "scriptextender",
];

pub fn classify_root_files(_staging_dir: &Path, relative_paths: &[String]) -> (Vec<RootFileEntry>, Vec<String>) {
    let mut root = Vec::new();
    let mut data = Vec::new();

    for rel in relative_paths {
        let lower = rel.to_lowercase();
        let file_name = Path::new(rel)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        let is_root = ROOT_NAMES.iter().any(|n| file_name.contains(n))
            || ROOT_EXTENSIONS.iter().any(|ext| lower.ends_with(&format!(".{ext}")))
            || lower.contains("script extender")
            || lower.contains("address library");

        if is_root {
            root.push(RootFileEntry {
                source: rel.clone(),
                target_name: Path::new(rel)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(rel)
                    .to_string(),
            });
        } else {
            data.push(rel.clone());
        }
    }

    (root, data)
}
