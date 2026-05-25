/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

mod bsa_fix;
mod loot;
mod plugins;
mod plugins_txt;

pub use bsa_fix::*;
pub use loot::*;
pub use plugins::*;
pub use plugins_txt::*;

use crate::games::GameProfile;

pub fn profile_supports_plugins(profile: &GameProfile) -> bool {
    profile.supports_plugins
}
