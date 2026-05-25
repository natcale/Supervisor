/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GamePlatform {
    Steam,
    Epic,
    Gog,
    Heroic,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedGame {
    pub id: String,
    pub name: String,
    pub platform: GamePlatform,
    pub install_path: String,
    pub executable: Option<String>,
    pub app_id: Option<String>,
    pub data_path: Option<String>,
    pub nexus_domain: Option<String>,
    #[serde(default)]
    pub profile_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameScanResult {
    pub games: Vec<DetectedGame>,
    pub scanned_at: u64,
}
