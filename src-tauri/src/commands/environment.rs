use crate::{
    error::AppError,
    process::SystemProcessRunner,
    services::environment::{EnvironmentService, EnvironmentStatus},
};

#[tauri::command]
pub(crate) async fn get_environment_status() -> Result<EnvironmentStatus, AppError> {
    tauri::async_runtime::spawn_blocking(|| EnvironmentService::new(SystemProcessRunner).inspect())
        .await
        .map_err(|_| AppError::environment_check_failed())
}
