/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
//! Run deploy-flow tests one engine family at a time (`cargo test deploy_flow_engine_`).

use supervisor_lib::testing::deploy_flow::engines;

macro_rules! engine_test {
    ($name:ident, $runner:expr) => {
        #[test]
        fn $name() {
            if let Err(report) = $runner() {
                panic!("{report}");
            }
        }
    };
}

engine_test!(deploy_flow_engine_bethesda, engines::bethesda);
engine_test!(deploy_flow_engine_data, engines::data);
engine_test!(deploy_flow_engine_kcd, engines::kcd);
engine_test!(deploy_flow_engine_cyberpunk, engines::cyberpunk);
engine_test!(deploy_flow_engine_bg3, engines::bg3);
engine_test!(deploy_flow_engine_mods, engines::mods);
engine_test!(deploy_flow_engine_mod_root, engines::mod_root);
engine_test!(deploy_flow_engine_stardew, engines::stardew);
engine_test!(deploy_flow_engine_bepinex, engines::bepinex);
engine_test!(deploy_flow_engine_subnautica, engines::subnautica);
engine_test!(deploy_flow_engine_marvel, engines::marvel);
engine_test!(deploy_flow_engine_unreal, engines::unreal);
engine_test!(deploy_flow_engine_mod_path, engines::mod_path);
engine_test!(deploy_flow_engine_game_root, engines::game_root);
