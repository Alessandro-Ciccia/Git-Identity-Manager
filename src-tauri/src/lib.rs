mod commands;
mod error;
mod process;
mod services;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_directory = app.path().app_data_dir().ok();
            let app_config_directory = app.path().app_config_dir().ok();
            let home_directory = app.path().home_dir().ok();
            app.manage(commands::profiles::ProfilesState::new(
                app_data_directory
                    .as_ref()
                    .map(|directory| directory.join("profiles.v1.json")),
            ));
            app.manage(commands::repositories::RepositoriesState::new(
                app_data_directory
                    .as_ref()
                    .map(|directory| directory.join("repositories.v1.json")),
            ));
            app.manage(commands::preferences::PreferencesState::new(
                app_data_directory
                    .as_ref()
                    .map(|directory| directory.join("preferences.v1.json")),
            ));
            app.manage(commands::directory_rules::DirectoryRulesState::new(
                app_data_directory.map(|directory| directory.join("directory-rules.v1.json")),
                app_config_directory
                    .as_ref()
                    .map(|directory| directory.join("git").join("identities")),
                app_config_directory.map(|directory| directory.join("git").join("backups")),
                home_directory
                    .map(|directory| {
                        vec![
                            directory.join(".gitconfig"),
                            directory.join(".config").join("git").join("config"),
                        ]
                    })
                    .unwrap_or_default(),
            ));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::directory_rules::list_directory_rules,
            commands::directory_rules::preview_directory_rule,
            commands::directory_rules::apply_directory_rule,
            commands::directory_rules::preview_remove_directory_rule,
            commands::directory_rules::remove_directory_rule,
            commands::environment::get_environment_status,
            commands::github::open_github_login_page,
            commands::github::open_github_account_page,
            commands::github::open_github_cli_update_page,
            commands::github::list_github_accounts,
            commands::github::switch_github_account,
            commands::github::launch_github_login,
            commands::preferences::get_preferences,
            commands::preferences::set_theme_preference,
            commands::preferences::set_welcome_dismissed,
            commands::profiles::list_profiles,
            commands::profiles::create_profile,
            commands::profiles::update_profile,
            commands::profiles::delete_profile,
            commands::repositories::list_repositories,
            commands::repositories::register_repository,
            commands::repositories::refresh_repository,
            commands::repositories::assign_repository_profile,
            commands::repositories::remove_repository_profile,
            commands::repositories::preview_repository_profile,
            commands::repositories::apply_repository_profile,
            commands::repositories::remove_repository,
            commands::repositories::reveal_repository,
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
