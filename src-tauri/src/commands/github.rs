use tauri_plugin_opener::OpenerExt;

use crate::{
    error::AppError,
    process::SystemProcessRunner,
    services::github_cli::{GithubAccountsStatus, GithubCliService},
};

const GITHUB_CLI_INSTALLATION_URL: &str = "https://github.com/cli/cli#installation";
const GITHUB_LOGIN_URL: &str = "https://github.com/login/device";

#[tauri::command]
pub(crate) fn open_github_login_page(app: tauri::AppHandle) -> Result<(), AppError> {
    app.opener()
        .open_url(GITHUB_LOGIN_URL, None::<&str>)
        .map_err(|_| AppError::github_login_page_open_failed())
}

#[tauri::command]
pub(crate) fn open_github_cli_update_page(app: tauri::AppHandle) -> Result<(), AppError> {
    app.opener()
        .open_url(GITHUB_CLI_INSTALLATION_URL, None::<&str>)
        .map_err(|_| AppError::external_page_open_failed())
}

#[tauri::command]
pub(crate) async fn open_github_account_page(
    app: tauri::AppHandle,
    hostname: String,
    username: String,
) -> Result<(), AppError> {
    let profile_url = tauri::async_runtime::spawn_blocking(move || {
        GithubCliService::new(SystemProcessRunner).account_profile_url(&hostname, &username)
    })
    .await
    .map_err(|_| AppError::github_operation_failed())?
    .map_err(AppError::from)?;

    app.opener()
        .open_url(profile_url, None::<&str>)
        .map_err(|_| AppError::github_account_page_open_failed())
}

#[tauri::command]
pub(crate) async fn list_github_accounts() -> Result<GithubAccountsStatus, AppError> {
    tauri::async_runtime::spawn_blocking(|| {
        GithubCliService::new(SystemProcessRunner).list_accounts_with_public_emails()
    })
    .await
    .map_err(|_| AppError::github_operation_failed())?
    .map_err(AppError::from)
}

#[tauri::command]
pub(crate) async fn switch_github_account(
    hostname: String,
    username: String,
) -> Result<GithubAccountsStatus, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        GithubCliService::new(SystemProcessRunner)
            .switch_account_with_public_emails(&hostname, &username)
    })
    .await
    .map_err(|_| AppError::github_operation_failed())?
    .map_err(AppError::from)
}

#[tauri::command]
pub(crate) async fn launch_github_login(
    hostname: String,
) -> Result<GithubAccountsStatus, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        GithubCliService::new(SystemProcessRunner).launch_login(&hostname)
    })
    .await
    .map_err(|_| AppError::github_operation_failed())?
    .map_err(AppError::from)
}
