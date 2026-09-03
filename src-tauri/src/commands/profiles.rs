use std::{path::PathBuf, sync::Mutex};

use crate::{
    error::AppError,
    services::profiles::{GitProfile, ProfileInput, ProfilesService},
};

pub(crate) struct ProfilesState {
    service: Mutex<ProfilesService>,
}

impl ProfilesState {
    pub(crate) fn new(store_path: Option<PathBuf>) -> Self {
        Self {
            service: Mutex::new(ProfilesService::new(store_path)),
        }
    }

    fn with_service<T>(
        &self,
        operation: impl FnOnce(&ProfilesService) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let service = self
            .service
            .lock()
            .map_err(|_| AppError::profile_operation_failed())?;
        operation(&service)
    }
}

#[tauri::command]
pub(crate) fn list_profiles(
    state: tauri::State<'_, ProfilesState>,
) -> Result<Vec<GitProfile>, AppError> {
    state.with_service(|service| service.list().map_err(AppError::from))
}

#[tauri::command]
pub(crate) fn create_profile(
    state: tauri::State<'_, ProfilesState>,
    profile: ProfileInput,
) -> Result<GitProfile, AppError> {
    state.with_service(|service| service.create(profile).map_err(AppError::from))
}

#[tauri::command]
pub(crate) fn update_profile(
    state: tauri::State<'_, ProfilesState>,
    id: String,
    profile: ProfileInput,
) -> Result<GitProfile, AppError> {
    state.with_service(|service| service.update(&id, profile).map_err(AppError::from))
}

#[tauri::command]
pub(crate) fn delete_profile(
    state: tauri::State<'_, ProfilesState>,
    id: String,
) -> Result<(), AppError> {
    state.with_service(|service| service.delete(&id).map_err(AppError::from))
}
