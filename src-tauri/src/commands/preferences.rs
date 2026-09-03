use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{
    error::AppError,
    services::preferences::{AppPreferences, PreferencesService, ThemePreference},
};

pub(crate) struct PreferencesState {
    service: Arc<Mutex<PreferencesService>>,
}

impl PreferencesState {
    pub(crate) fn new(store_path: Option<PathBuf>) -> Self {
        Self {
            service: Arc::new(Mutex::new(PreferencesService::new(store_path))),
        }
    }

    fn with_service<T>(
        &self,
        operation: impl FnOnce(&PreferencesService) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let service = self
            .service
            .lock()
            .map_err(|_| AppError::preference_operation_failed())?;
        operation(&service)
    }
}

#[tauri::command]
pub(crate) fn get_preferences(
    state: tauri::State<'_, PreferencesState>,
) -> Result<AppPreferences, AppError> {
    state.with_service(|service| service.get().map_err(AppError::from))
}

#[tauri::command]
pub(crate) fn set_theme_preference(
    state: tauri::State<'_, PreferencesState>,
    theme: ThemePreference,
) -> Result<AppPreferences, AppError> {
    state.with_service(|service| service.set_theme(theme).map_err(AppError::from))
}

#[tauri::command]
pub(crate) fn set_welcome_dismissed(
    state: tauri::State<'_, PreferencesState>,
    dismissed: bool,
) -> Result<AppPreferences, AppError> {
    state.with_service(|service| {
        service
            .set_welcome_dismissed(dismissed)
            .map_err(AppError::from)
    })
}
