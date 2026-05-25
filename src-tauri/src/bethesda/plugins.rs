/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginEntry {
    pub name: String,
    pub mod_id: String,
    pub enabled: bool,
    pub is_master: bool,
}

const PLUGIN_EXTS: &[&str] = &["esp", "esm", "esl"];

pub fn is_plugin(path: &str) -> bool {
    path.rsplit('.')
        .next()
        .map(|ext| PLUGIN_EXTS.iter().any(|e| ext.eq_ignore_ascii_case(e)))
        .unwrap_or(false)
}

pub fn is_master(path: &str) -> bool {
    path.rsplit('.')
        .next()
        .map(|ext| ext.eq_ignore_ascii_case("esm"))
        .unwrap_or(false)
}

pub fn scan_plugins_from_mods(
    mods: &[(String, Vec<String>)],
    enabled_ids: &[String],
) -> Vec<PluginEntry> {
    let enabled: std::collections::HashSet<_> = enabled_ids.iter().cloned().collect();
    let mut plugins = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for (mod_id, files) in mods {
        if !enabled.contains(mod_id) {
            continue;
        }
        for file in files {
            if !is_plugin(file) {
                continue;
            }
            let name = file
                .rsplit(['/', '\\'])
                .next()
                .unwrap_or(file)
                .to_string();
            if seen.insert(name.clone()) {
                plugins.push(PluginEntry {
                    name,
                    mod_id: mod_id.clone(),
                    enabled: true,
                    is_master: is_master(file),
                });
            }
        }
    }

    plugins.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    plugins
}

pub fn apply_load_order(plugins: &mut [PluginEntry], order: &[String]) {
    let rank: std::collections::HashMap<_, _> = order
        .iter()
        .enumerate()
        .map(|(i, n)| (n.to_lowercase(), i))
        .collect();
    plugins.sort_by_key(|p| rank.get(&p.name.to_lowercase()).copied().unwrap_or(usize::MAX));
}
