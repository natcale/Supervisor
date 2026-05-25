/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
mod executor;
mod parser;

pub use executor::apply_fomod_selection;
pub use parser::{parse_fomod_config, FomodConfig};

use crate::install::installers::find_module_config;
use std::path::Path;

pub fn has_fomod(mod_root: &Path) -> bool {
    find_module_config(mod_root).is_some()
}
