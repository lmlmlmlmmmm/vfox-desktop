//! vfox-desktop 后端入口
//!
//! 见 `src-tauri/src/main.rs`，bin 仅调用 [`run`]。

pub mod commands;
pub mod config;
pub mod error;
pub mod paths;
pub mod vfox;

use commands::{config as cmd_config, plugin, sdk, system, updater};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 启动时清理上次自更新残留的 *.old / *.new（Windows 运行中的 exe 无法删，
    // 只能延迟到下次启动）
    updater::cleanup_stale_old_exe();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // system
            system::check_vfox_installed,
            system::open_vfox_home,
            // sdk
            sdk::list_sdks,
            sdk::current_versions,
            sdk::search_versions,
            sdk::use_sdk,
            sdk::uninstall_sdk,
            sdk::install_sdk,
            // plugin
            plugin::list_available_plugins,
            plugin::plugin_info,
            plugin::add_plugin,
            plugin::remove_plugin,
            plugin::update_plugin,
            plugin::update_all_plugins,
            // config
            cmd_config::get_config,
            cmd_config::save_config,
            // updater (app self-update)
            updater::app_version,
            updater::check_app_update,
            updater::download_and_apply_app_update,
        ])
        .run(tauri::generate_context!())
        .expect("vfox-desktop 启动失败");
}
