/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
//! Nexus lookup — backed by `game_profiles.json` via `profile_loader`.

pub use super::profile_loader::{
    mod_path_hint, nexus_domain_for_steam, profile_id_for_domain, profile_id_for_steam,
};
