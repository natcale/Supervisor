/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
mod bethesda;

mod bg3;

mod collections;

mod commands;

mod deep_link;

mod cyberpunk;

mod deploy;

mod hardlink;

mod diagnostics;

mod errors;

mod game_detection;

mod games;

mod ingest;

mod install;

mod library;

mod loadouts;

mod nexus;

mod secrets;

mod settings;

mod themes;

mod root_builder;

mod vdf;

use commands::handle_argv;

use nexus::DownloadQueue;

use std::sync::Arc;

use tauri::Manager;

use tauri_plugin_deep_link::DeepLinkExt;

fn focus_main_window_if_needed(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    let minimized = window.is_minimized().unwrap_or(false);

    let visible = window.is_visible().unwrap_or(true);

    if minimized || !visible {
        let _ = window.unminimize();

        let _ = window.show();

        let _ = window.set_focus();
    }
}

pub struct AppState {
    pub downloads: Arc<DownloadQueue>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]

pub fn run() {
    let downloads = Arc::new(DownloadQueue::new());

    let mut builder = tauri::Builder::default()
        .manage(AppState { downloads })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_process::init());

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    }

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            handle_argv(app, &argv);

            focus_main_window_if_needed(app);
        }));
    }

    builder
        .setup(|app| {
            let app_data = app.path().app_data_dir()?;
            let _ = secrets::migrate_nexus_api_key_from_settings(&app_data);
            let settings = settings::load_settings(&app_data).unwrap_or_default();

            let log_level = if settings.debug_logging {
                log::LevelFilter::Debug
            } else if cfg!(debug_assertions) {
                log::LevelFilter::Info
            } else {
                log::LevelFilter::Warn
            };
            let _ = app.handle().plugin(
                tauri_plugin_log::Builder::default()
                    .level(log_level)
                    .build(),
            );

            #[cfg(desktop)]
            {
                app.deep_link().register("nxm")?;
            }

            #[cfg(desktop)]
            {
                let handle = app.handle().clone();
                app.deep_link().on_open_url(move |event| {
                    for url in event.urls() {
                        if let Some(payload) = deep_link::parse_nxm_url(url.as_str()) {
                            commands::handle_nxm_payload(&handle, payload);
                            focus_main_window_if_needed(&handle);
                        }
                    }
                });
            }

            if let Some(argv) = std::env::args().collect::<Vec<_>>().get(1..) {
                handle_argv(app.handle(), argv);
            }

            settings::ensure_app_dirs(&app_data).ok();
            app.state::<AppState>().downloads.init(&app_data)?;
            let _ = themes::ensure_themes_dir(&app_data);
            commands::downloads::pump_download_queue(
                app.handle(),
                &app.state::<AppState>().downloads,
            );

            // Completed setup should go straight to the main window
            if settings.onboarding_complete {
                if let Some(onboarding) = app.get_webview_window("onboarding") {
                    let _ = onboarding.close();
                }
                if let Some(main) = app.get_webview_window("main") {
                    let _ = main.show();
                    let _ = main.set_focus();
                }
            } else {
                if let Some(main) = app.get_webview_window("main") {
                    let _ = main.hide();
                }
                if let Some(onboarding) = app.get_webview_window("onboarding") {
                    let _ = onboarding.show();
                    let _ = onboarding.set_focus();
                }
            }

            commands::configure_runtime_from_settings(app.handle(), &settings);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::scan_games,
            commands::check_partition,
            commands::deploy_game_mods,
            commands::undeploy_mod,
            commands::fix_bsa_timestamps,
            commands::get_deploy_state,
            commands::run_preflight_checks,
            commands::get_deploy_targets,
            commands::get_game_profile,
            commands::list_supported_profiles,
            commands::purge_deployed_mods,
            commands::download_nxm_mod,
            commands::parse_nxm,
            commands::get_staging_dir,
            commands::ingest_mod_paths,
            commands::launch_game,
            commands::get_library,
            commands::remove_library_mod,
            commands::reorder_library_mods,
            commands::parse_fomod,
            commands::apply_fomod,
            commands::list_loadouts,
            commands::switch_loadout,
            commands::create_loadout,
            commands::update_loadout,
            commands::delete_loadout,
            commands::get_download_queue,
            commands::enqueue_nxm_download,
            commands::start_pending_downloads,
            commands::cancel_download,
            commands::get_plugin_list,
            commands::sort_plugins_loot,
            commands::set_plugin_order,
            commands::toggle_plugin,
            commands::add_manual_game,
            commands::remove_manual_game,
            commands::update_manual_game_nexus_domain,
            commands::import_vortex_collection,
            commands::install_collection_mods,
            commands::get_game_state,
            commands::get_app_settings,
            commands::update_app_settings,
            commands::get_app_paths,
            commands::open_path,
            commands::complete_onboarding,
            commands::open_staging_folder,
            commands::open_mod_folder,
            commands::reinstall_mod,
            commands::fetch_nexus_mod_metadata,
            commands::check_mod_updates,
            commands::enrich_mod_metadata,
            commands::validate_nexus_api_key,
            commands::clear_failed_downloads,
            commands::clear_finished_downloads,
            commands::set_mod_notes,
            commands::list_themes,
            commands::load_active_theme,
            commands::set_active_theme,
            commands::install_theme,
            commands::get_platform,
            commands::read_theme_asset,
            commands::open_themes_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
