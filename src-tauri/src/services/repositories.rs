use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use super::git::RepositoryInspection;

const STORE_VERSION: u32 = 1;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RepositoryRegistration {
    pub(crate) id: String,
    pub(crate) path: String,
    pub(crate) added_at: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RegisteredRepository {
    pub(crate) id: String,
    pub(crate) path: String,
    pub(crate) added_at: String,
    pub(crate) state: RepositoryState,
    pub(crate) inspection: Option<RepositoryInspection>,
    pub(crate) message: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum RepositoryState {
    Available,
    Missing,
    Invalid,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RepositoryError {
    StorageUnavailable,
    StorageReadFailed,
    StorageWriteFailed,
    MalformedData,
    InvalidId,
    InvalidPath,
    NotFound,
    ClockUnavailable,
}

pub(crate) struct RepositoriesService {
    store_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct RepositoriesStore {
    version: u32,
    repositories: Vec<RepositoryRegistration>,
}

impl RepositoriesService {
    pub(crate) fn new(store_path: Option<PathBuf>) -> Self {
        Self { store_path }
    }

    pub(crate) fn list(&self) -> Result<Vec<RepositoryRegistration>, RepositoryError> {
        Ok(self.load()?.repositories)
    }

    pub(crate) fn find(&self, id: &str) -> Result<RepositoryRegistration, RepositoryError> {
        validate_id(id)?;
        self.load()?
            .repositories
            .into_iter()
            .find(|repository| repository.id == id)
            .ok_or(RepositoryError::NotFound)
    }

    pub(crate) fn register(&self, path: &str) -> Result<RepositoryRegistration, RepositoryError> {
        validate_path(path)?;
        let mut store = self.load()?;
        if let Some(existing) = store
            .repositories
            .iter()
            .find(|repository| repository.path == path)
        {
            return Ok(existing.clone());
        }

        let registration = RepositoryRegistration {
            id: Uuid::new_v4().to_string(),
            path: path.to_owned(),
            added_at: OffsetDateTime::now_utc()
                .format(&Rfc3339)
                .map_err(|_| RepositoryError::ClockUnavailable)?,
        };
        store.repositories.push(registration.clone());
        self.save(&store)?;

        self.find(&registration.id)
            .map_err(|_| RepositoryError::StorageWriteFailed)
    }

    pub(crate) fn remove(&self, id: &str) -> Result<(), RepositoryError> {
        validate_id(id)?;
        let mut store = self.load()?;
        let original_len = store.repositories.len();
        store.repositories.retain(|repository| repository.id != id);
        if store.repositories.len() == original_len {
            return Err(RepositoryError::NotFound);
        }

        self.save(&store)?;
        if self
            .load()?
            .repositories
            .iter()
            .any(|repository| repository.id == id)
        {
            return Err(RepositoryError::StorageWriteFailed);
        }
        Ok(())
    }

    fn load(&self) -> Result<RepositoriesStore, RepositoryError> {
        let path = self.path()?;
        recover_backup_if_needed(path)?;
        if !path.exists() {
            return Ok(RepositoriesStore {
                version: STORE_VERSION,
                repositories: Vec::new(),
            });
        }

        let contents = fs::read(path).map_err(|_| RepositoryError::StorageReadFailed)?;
        let store: RepositoriesStore =
            serde_json::from_slice(&contents).map_err(|_| RepositoryError::MalformedData)?;
        validate_store(&store)?;
        Ok(store)
    }

    fn save(&self, store: &RepositoriesStore) -> Result<(), RepositoryError> {
        let path = self.path()?;
        let parent = path.parent().ok_or(RepositoryError::StorageUnavailable)?;
        fs::create_dir_all(parent).map_err(|_| RepositoryError::StorageWriteFailed)?;
        let contents =
            serde_json::to_vec_pretty(store).map_err(|_| RepositoryError::StorageWriteFailed)?;
        let temporary_path = parent.join(format!(
            ".{}.{}.tmp",
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("repositories"),
            Uuid::new_v4()
        ));

        let result = (|| {
            let mut temporary = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temporary_path)
                .map_err(|_| RepositoryError::StorageWriteFailed)?;
            temporary
                .write_all(&contents)
                .and_then(|_| temporary.sync_all())
                .map_err(|_| RepositoryError::StorageWriteFailed)?;
            replace_with_backup(path, &temporary_path)
        })();

        if result.is_err() {
            let _ = fs::remove_file(&temporary_path);
        }
        result
    }

    fn path(&self) -> Result<&Path, RepositoryError> {
        self.store_path
            .as_deref()
            .ok_or(RepositoryError::StorageUnavailable)
    }
}

fn validate_store(store: &RepositoriesStore) -> Result<(), RepositoryError> {
    if store.version != STORE_VERSION {
        return Err(RepositoryError::MalformedData);
    }

    let mut ids = HashSet::new();
    let mut paths = HashSet::new();
    for repository in &store.repositories {
        validate_id(&repository.id).map_err(|_| RepositoryError::MalformedData)?;
        validate_path(&repository.path).map_err(|_| RepositoryError::MalformedData)?;
        if OffsetDateTime::parse(&repository.added_at, &Rfc3339).is_err()
            || !ids.insert(repository.id.as_str())
            || !paths.insert(repository.path.as_str())
        {
            return Err(RepositoryError::MalformedData);
        }
    }
    Ok(())
}

fn validate_id(id: &str) -> Result<(), RepositoryError> {
    Uuid::parse_str(id)
        .map(|_| ())
        .map_err(|_| RepositoryError::InvalidId)
}

fn validate_path(path: &str) -> Result<(), RepositoryError> {
    if path.is_empty()
        || path.chars().count() > 32_768
        || path.contains('\0')
        || !Path::new(path).is_absolute()
    {
        Err(RepositoryError::InvalidPath)
    } else {
        Ok(())
    }
}

fn backup_path(path: &Path) -> PathBuf {
    path.with_extension("json.backup")
}

fn recover_backup_if_needed(path: &Path) -> Result<(), RepositoryError> {
    let backup = backup_path(path);
    if !path.exists() && backup.exists() {
        fs::rename(backup, path).map_err(|_| RepositoryError::StorageReadFailed)?;
    }
    Ok(())
}

fn replace_with_backup(path: &Path, temporary_path: &Path) -> Result<(), RepositoryError> {
    let backup = backup_path(path);
    if backup.exists() {
        fs::remove_file(&backup).map_err(|_| RepositoryError::StorageWriteFailed)?;
    }

    let had_existing = path.exists();
    if had_existing {
        fs::rename(path, &backup).map_err(|_| RepositoryError::StorageWriteFailed)?;
    }
    if fs::rename(temporary_path, path).is_err() {
        if had_existing {
            let _ = fs::rename(&backup, path);
        }
        return Err(RepositoryError::StorageWriteFailed);
    }
    if had_existing {
        let _ = fs::remove_file(backup);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "git-identity-manager-repositories-test-{}",
                Uuid::new_v4()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn service(directory: &Path) -> RepositoriesService {
        RepositoriesService::new(Some(directory.join("repositories.v1.json")))
    }

    #[test]
    fn missing_store_is_empty_without_creating_a_file() {
        let directory = TestDirectory::new();
        assert!(service(&directory.0).list().unwrap().is_empty());
        assert!(!directory.0.join("repositories.v1.json").exists());
    }

    #[test]
    fn registrations_are_unique_and_survive_restart() {
        let directory = TestDirectory::new();
        let repository_path = directory.0.join("project").to_string_lossy().into_owned();
        let first = service(&directory.0).register(&repository_path).unwrap();
        let duplicate = service(&directory.0).register(&repository_path).unwrap();

        assert_eq!(first, duplicate);
        assert!(Uuid::parse_str(&first.id).is_ok());
        assert!(OffsetDateTime::parse(&first.added_at, &Rfc3339).is_ok());
        assert_eq!(service(&directory.0).list().unwrap(), vec![first]);
    }

    #[test]
    fn removal_persists_and_does_not_touch_repository_path() {
        let directory = TestDirectory::new();
        let repository = directory.0.join("actual-repository");
        fs::create_dir(&repository).unwrap();
        let path = repository.to_string_lossy().into_owned();
        let registration = service(&directory.0).register(&path).unwrap();

        service(&directory.0).remove(&registration.id).unwrap();

        assert!(repository.exists());
        assert!(service(&directory.0).list().unwrap().is_empty());
    }

    #[test]
    fn rejects_bad_ids_and_malformed_stores() {
        let directory = TestDirectory::new();
        assert_eq!(
            service(&directory.0).find("bad-id"),
            Err(RepositoryError::InvalidId)
        );

        fs::write(
            directory.0.join("repositories.v1.json"),
            r#"{"version":2,"repositories":[]}"#,
        )
        .unwrap();
        assert_eq!(
            service(&directory.0).list(),
            Err(RepositoryError::MalformedData)
        );
    }
}
