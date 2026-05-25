/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeMode {
    Flat,
    PerModFolder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModTypeDef {
    pub id: String,
    pub rel_path: String,
    pub merge: bool,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementDef {
    pub id: String,
    pub label: String,
    pub path: String,
    pub optional: bool,
    pub create_if_missing: bool,
}

#[derive(Debug, Clone)]
pub struct GameProfile {
    pub id: String,
    pub name: String,
    pub nexus_domains: Vec<String>,
    pub steam_app_ids: Vec<String>,
    pub mod_types: Vec<ModTypeDef>,
    pub merge_mode: MergeMode,
    pub requirements: Vec<RequirementDef>,
    pub supports_plugins: bool,
}

impl GameProfile {
    pub fn default_mod_type(&self) -> &ModTypeDef {
        if let Some(t) = self.mod_types.iter().find(|t| t.id == "default") {
            return t;
        }
        self.mod_types
            .iter()
            .min_by_key(|t| t.priority)
            .expect("profile must define mod types")
    }

    pub fn mod_type(&self, id: &str) -> Option<&ModTypeDef> {
        self.mod_types.iter().find(|t| t.id == id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameProfileSummary {
    pub id: String,
    pub name: String,
    pub primary_mod_path: String,
    pub is_generic: bool,
    pub supports_plugins: bool,
    #[serde(default)]
    pub nexus_domains: Vec<String>,
    #[serde(default)]
    pub steam_app_ids: Vec<String>,
}
