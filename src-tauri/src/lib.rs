mod commands;
mod error;
mod process;
mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::environment::get_environment_status,
            commands::github::open_github_login_page,
            commands::github::open_github_account_page,
            commands::github::open_github_cli_update_page,
            commands::github::list_github_accounts,
            commands::github::switch_github_account,
            commands::github::launch_github_login,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    #[test]
    fn bootstrap_test_harness_is_available() {
        assert_eq!("git-identity-manager", env!("CARGO_PKG_NAME"));
    }
}
