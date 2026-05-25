/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
//! Game deploy orchestration: manifest, verify, purge, and preflight.
pub mod manifest;
mod engine;
mod purge;
mod requirements;
mod sync;
mod targets;
mod verify;

pub use engine::*;
pub use purge::*;
pub use sync::prune_deploy_manifest;
