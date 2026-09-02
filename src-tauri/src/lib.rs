mod commands;
mod error;
mod process;
mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::environment::get_environment_status
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
