use std::sync::{Arc, Mutex};

use crate::{
    error::AppError,
    process::SystemProcessRunner,
    services::directory_rules::{
        DirectoryRuleInput, DirectoryRulePreview, DirectoryRulesService, ResolvedDirectoryRule,
    },
};

use super::{profiles::ProfilesState, repositories::RepositoriesState};

pub(crate) struct DirectoryRulesState {
    service: Arc<Mutex<DirectoryRulesService<SystemProcessRunner>>>,
}

impl DirectoryRulesState {
    pub(crate) fn new(
        store_path: Option<std::path::PathBuf>,
        identity_directory: Option<std::path::PathBuf>,
        backup_directory: Option<std::path::PathBuf>,
        global_config_candidates: Vec<std::path::PathBuf>,
    ) -> Self {
        Self {
            service: Arc::new(Mutex::new(DirectoryRulesService::new(
                store_path,
                identity_directory,
                backup_directory,
                global_config_candidates,
                SystemProcessRunner,
            ))),
        }
    }

    fn service(&self) -> Arc<Mutex<DirectoryRulesService<SystemProcessRunner>>> {
        Arc::clone(&self.service)
    }
}

#[tauri::command]
pub(crate) async fn list_directory_rules(
    state: tauri::State<'_, DirectoryRulesState>,
    profiles_state: tauri::State<'_, ProfilesState>,
) -> Result<Vec<ResolvedDirectoryRule>, AppError> {
    let rules = state.service();
    let profiles = profiles_state.service();
    tauri::async_runtime::spawn_blocking(move || {
        let profiles = profiles
            .lock()
            .map_err(|_| AppError::directory_rule_operation_failed())?;
        rules
            .lock()
            .map_err(|_| AppError::directory_rule_operation_failed())?
            .list(&profiles)
            .map_err(AppError::from)
    })
    .await
    .map_err(|_| AppError::directory_rule_operation_failed())?
}

#[tauri::command]
pub(crate) async fn preview_directory_rule(
    state: tauri::State<'_, DirectoryRulesState>,
    profiles_state: tauri::State<'_, ProfilesState>,
    repositories_state: tauri::State<'_, RepositoriesState>,
    input: DirectoryRuleInput,
) -> Result<DirectoryRulePreview, AppError> {
    let rules = state.service();
    let profiles = profiles_state.service();
    let repositories = repositories_state.service();
    tauri::async_runtime::spawn_blocking(move || {
        let profiles = profiles
            .lock()
            .map_err(|_| AppError::directory_rule_operation_failed())?;
        let repositories = repositories
            .lock()
            .map_err(|_| AppError::directory_rule_operation_failed())?;
        rules
            .lock()
            .map_err(|_| AppError::directory_rule_operation_failed())?
            .preview(input, &profiles, &repositories)
            .map_err(AppError::from)
    })
    .await
    .map_err(|_| AppError::directory_rule_operation_failed())?
}

#[tauri::command]
pub(crate) async fn apply_directory_rule(
    state: tauri::State<'_, DirectoryRulesState>,
    profiles_state: tauri::State<'_, ProfilesState>,
    repositories_state: tauri::State<'_, RepositoriesState>,
    input: DirectoryRuleInput,
) -> Result<ResolvedDirectoryRule, AppError> {
    let rules = state.service();
    let profiles = profiles_state.service();
    let repositories = repositories_state.service();
    tauri::async_runtime::spawn_blocking(move || {
        let profiles = profiles
            .lock()
            .map_err(|_| AppError::directory_rule_operation_failed())?;
        let repositories = repositories
            .lock()
            .map_err(|_| AppError::directory_rule_operation_failed())?;
        rules
            .lock()
            .map_err(|_| AppError::directory_rule_operation_failed())?
            .apply(input, &profiles, &repositories)
            .map_err(AppError::from)
    })
    .await
    .map_err(|_| AppError::directory_rule_operation_failed())?
}

#[tauri::command]
pub(crate) async fn preview_remove_directory_rule(
    state: tauri::State<'_, DirectoryRulesState>,
    profiles_state: tauri::State<'_, ProfilesState>,
    id: String,
) -> Result<DirectoryRulePreview, AppError> {
    let rules = state.service();
    let profiles = profiles_state.service();
    tauri::async_runtime::spawn_blocking(move || {
        let profiles = profiles
            .lock()
            .map_err(|_| AppError::directory_rule_operation_failed())?;
        rules
            .lock()
            .map_err(|_| AppError::directory_rule_operation_failed())?
            .preview_remove(&id, &profiles)
            .map_err(AppError::from)
    })
    .await
    .map_err(|_| AppError::directory_rule_operation_failed())?
}

#[tauri::command]
pub(crate) async fn remove_directory_rule(
    state: tauri::State<'_, DirectoryRulesState>,
    id: String,
) -> Result<(), AppError> {
    let rules = state.service();
    tauri::async_runtime::spawn_blocking(move || {
        rules
            .lock()
            .map_err(|_| AppError::directory_rule_operation_failed())?
            .remove(&id)
            .map_err(AppError::from)
    })
    .await
    .map_err(|_| AppError::directory_rule_operation_failed())?
}
