/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
//! Full deploy → verify hardlinks → undeploy → purge cycle for every built-in game profile.
//! Sandboxes live under the OS temp directory, not in this repository.

use supervisor_lib::testing::deploy_flow::run_all;

#[test]
fn deploy_flow_run_all_profiles() {
    if let Err(report) = run_all() {
        panic!("{report}");
    }
}
