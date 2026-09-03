use std::sync::{Arc, Mutex};

use tauri_plugin_opener::OpenerExt;

use crate::{
    error::AppError,
    process::SystemProcessRunner,
    services::{
        git::{GitInspectionError, GitService, RepositoryInspection},
        repositories::{
            RegisteredRepository, RepositoriesService, RepositoryRegistration, RepositoryState,
        },
        repository_assignment::{RepositoryAssignmentService, RepositoryProfilePreview},
    },
};

use super::profiles::ProfilesState;

pub(crate) struct RepositoriesState {
    service: Arc<Mutex<RepositoriesService>>,
}

impl RepositoriesState {
    pub(crate) fn new(store_path: Option<std::path::PathBuf>) -> Self {
        Self {
            service: Arc::new(Mutex::new(RepositoriesService::new(store_path))),
        }
    }

    pub(crate) fn service(&self) -> Arc<Mutex<RepositoriesService>> {
        Arc::clone(&self.service)
    }
}

#[tauri::command]
pub(crate) async fn list_repositories(
    state: tauri::State<'_, RepositoriesState>,
) -> Result<Vec<RegisteredRepository>, AppError> {
    let service = state.service();
    tauri::async_runtime::spawn_blocking(move || {
        let registrations = service
            .lock()
            .map_err(|_| AppError::repository_operation_failed())?
            .list()
            .map_err(AppError::from)?;
        Ok(registrations
            .into_iter()
            .map(inspect_registration)
            .collect())
    })
    .await
    .map_err(|_| AppError::repository_operation_failed())?
}

#[tauri::command]
pub(crate) async fn register_repository(
    state: tauri::State<'_, RepositoriesState>,
    path: String,
) -> Result<RegisteredRepository, AppError> {
    let service = state.service();
    tauri::async_runtime::spawn_blocking(move || {
        let inspection = GitService::new(SystemProcessRunner)
            .inspect(&path)
            .map_err(AppError::from)?;
        let registration = service
            .lock()
            .map_err(|_| AppError::repository_operation_failed())?
            .register(&inspection.path)
            .map_err(AppError::from)?;
        Ok(available_repository(registration, inspection))
    })
    .await
    .map_err(|_| AppError::repository_operation_failed())?
}

#[tauri::command]
pub(crate) async fn refresh_repository(
    state: tauri::State<'_, RepositoriesState>,
    id: String,
) -> Result<RegisteredRepository, AppError> {
    let service = state.service();
    tauri::async_runtime::spawn_blocking(move || {
        let registration = service
            .lock()
            .map_err(|_| AppError::repository_operation_failed())?
            .find(&id)
            .map_err(AppError::from)?;
        Ok(inspect_registration(registration))
    })
    .await
    .map_err(|_| AppError::repository_operation_failed())?
}

#[tauri::command]
pub(crate) async fn assign_repository_profile(
    state: tauri::State<'_, RepositoriesState>,
    profiles_state: tauri::State<'_, ProfilesState>,
    id: String,
    profile_id: String,
) -> Result<RegisteredRepository, AppError> {
    let repositories = state.service();
    let profiles = profiles_state.service();
    tauri::async_runtime::spawn_blocking(move || {
        let profiles = profiles
            .lock()
            .map_err(|_| AppError::repository_assignment_failed())?;
        let repositories = repositories
            .lock()
            .map_err(|_| AppError::repository_assignment_failed())?;
        let registration =
            RepositoryAssignmentService::new(&repositories, &profiles, SystemProcessRunner)
                .assign(&id, &profile_id)
                .map_err(AppError::from)?;
        Ok(inspect_registration(registration))
    })
    .await
    .map_err(|_| AppError::repository_assignment_failed())?
}

#[tauri::command]
pub(crate) async fn remove_repository_profile(
    state: tauri::State<'_, RepositoriesState>,
    profiles_state: tauri::State<'_, ProfilesState>,
    id: String,
) -> Result<RegisteredRepository, AppError> {
    let repositories = state.service();
    let profiles = profiles_state.service();
    tauri::async_runtime::spawn_blocking(move || {
        let profiles = profiles
            .lock()
            .map_err(|_| AppError::repository_assignment_failed())?;
        let repositories = repositories
            .lock()
            .map_err(|_| AppError::repository_assignment_failed())?;
        let registration =
            RepositoryAssignmentService::new(&repositories, &profiles, SystemProcessRunner)
                .unassign(&id)
                .map_err(AppError::from)?;
        Ok(inspect_registration(registration))
    })
    .await
    .map_err(|_| AppError::repository_assignment_failed())?
}

#[tauri::command]
pub(crate) async fn preview_repository_profile(
    state: tauri::State<'_, RepositoriesState>,
    profiles_state: tauri::State<'_, ProfilesState>,
    id: String,
) -> Result<RepositoryProfilePreview, AppError> {
    let repositories = state.service();
    let profiles = profiles_state.service();
    tauri::async_runtime::spawn_blocking(move || {
        let profiles = profiles
            .lock()
            .map_err(|_| AppError::repository_assignment_failed())?;
        let repositories = repositories
            .lock()
            .map_err(|_| AppError::repository_assignment_failed())?;
        RepositoryAssignmentService::new(&repositories, &profiles, SystemProcessRunner)
            .preview(&id)
            .map_err(AppError::from)
    })
    .await
    .map_err(|_| AppError::repository_assignment_failed())?
}

#[tauri::command]
pub(crate) async fn apply_repository_profile(
    state: tauri::State<'_, RepositoriesState>,
    profiles_state: tauri::State<'_, ProfilesState>,
    id: String,
) -> Result<RegisteredRepository, AppError> {
    let repositories = state.service();
    let profiles = profiles_state.service();
    tauri::async_runtime::spawn_blocking(move || {
        let profiles = profiles
            .lock()
            .map_err(|_| AppError::repository_assignment_failed())?;
        let repositories = repositories
            .lock()
            .map_err(|_| AppError::repository_assignment_failed())?;
        let result =
            RepositoryAssignmentService::new(&repositories, &profiles, SystemProcessRunner)
                .apply(&id)
                .map_err(AppError::from)?;
        Ok(available_repository(result.registration, result.inspection))
    })
    .await
    .map_err(|_| AppError::repository_assignment_failed())?
}

#[tauri::command]
pub(crate) async fn remove_repository(
    state: tauri::State<'_, RepositoriesState>,
    id: String,
) -> Result<(), AppError> {
    let service = state.service();
    tauri::async_runtime::spawn_blocking(move || {
        service
            .lock()
            .map_err(|_| AppError::repository_operation_failed())?
            .remove(&id)
            .map_err(AppError::from)
    })
    .await
    .map_err(|_| AppError::repository_operation_failed())?
}

#[tauri::command]
pub(crate) async fn reveal_repository(
    app: tauri::AppHandle,
    state: tauri::State<'_, RepositoriesState>,
    id: String,
) -> Result<(), AppError> {
    let service = state.service();
    let registration = tauri::async_runtime::spawn_blocking(move || {
        service
            .lock()
            .map_err(|_| AppError::repository_operation_failed())?
            .find(&id)
            .map_err(AppError::from)
    })
    .await
    .map_err(|_| AppError::repository_operation_failed())??;

    let canonical =
        dunce::canonicalize(&registration.path).map_err(|_| AppError::repository_path_missing())?;
    if !canonical.is_dir() || canonical.to_str() != Some(registration.path.as_str()) {
        return Err(AppError::repository_path_missing());
    }

    app.opener()
        .open_path(registration.path, None::<&str>)
        .map_err(|_| AppError::repository_reveal_failed())
}

fn inspect_registration(registration: RepositoryRegistration) -> RegisteredRepository {
    match GitService::new(SystemProcessRunner).inspect(&registration.path) {
        Ok(inspection) => available_repository(registration, inspection),
        Err(error) => unavailable_repository(registration, error),
    }
}

fn available_repository(
    registration: RepositoryRegistration,
    inspection: RepositoryInspection,
) -> RegisteredRepository {
    RegisteredRepository {
        id: registration.id,
        path: registration.path,
        added_at: registration.added_at,
        profile_id: registration.profile_id,
        state: RepositoryState::Available,
        inspection: Some(inspection),
        message: None,
    }
}

fn unavailable_repository(
    registration: RepositoryRegistration,
    error: GitInspectionError,
) -> RegisteredRepository {
    let (state, message) = match error {
        GitInspectionError::PathNotFound | GitInspectionError::NotDirectory => (
            RepositoryState::Missing,
            "This repository folder is missing or has moved.",
        ),
        GitInspectionError::NotRepository => (
            RepositoryState::Invalid,
            "This folder is no longer a Git working repository.",
        ),
        GitInspectionError::GitMissing => (
            RepositoryState::Unavailable,
            "Git is unavailable, so this repository cannot be inspected.",
        ),
        GitInspectionError::InvalidPath
        | GitInspectionError::CommandFailed
        | GitInspectionError::MalformedOutput => (
            RepositoryState::Unavailable,
            "This repository could not be inspected safely.",
        ),
    };

    RegisteredRepository {
        id: registration.id,
        path: registration.path,
        added_at: registration.added_at,
        profile_id: registration.profile_id,
        state,
        inspection: None,
        message: Some(message.to_owned()),
    }
}
