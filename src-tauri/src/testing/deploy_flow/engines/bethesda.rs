/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::testing::deploy_flow::{run_engine, EngineKey, FlowReport};

pub fn run() -> Result<(), FlowReport> {
    run_engine(EngineKey::Bethesda)
}
