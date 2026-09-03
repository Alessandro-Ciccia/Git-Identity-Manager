use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

const STORE_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct AppPreferences {
    pub(crate) theme: ThemePreference,
    pub(crate) welcome_dismissed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PreferenceError {
    StorageUnavailable,
    StorageReadFailed,
    StorageWriteFailed,
    MalformedData,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PreferencesStore {
    version: u32,
    preferences: AppPreferences,
}

pub(crate) struct PreferencesService {
    store_path: Option<PathBuf>,
}

impl PreferencesService {
    pub(crate) fn new(store_path: Option<PathBuf>) -> Self {
        Self { store_path }
    }

    pub(crate) fn get(&self) -> Result<AppPreferences, PreferenceError> {
        Ok(self.load()?.preferences)
    }

    pub(crate) fn set_theme(
        &self,
        theme: ThemePreference,
    ) -> Result<AppPreferences, PreferenceError> {
        self.update(|preferences| preferences.theme = theme)
    }

    pub(crate) fn set_welcome_dismissed(
        &self,
        dismissed: bool,
    ) -> Result<AppPreferences, PreferenceError> {
        self.update(|preferences| preferences.welcome_dismissed = dismissed)
    }

    fn update(
        &self,
        mutate: impl FnOnce(&mut AppPreferences),
    ) -> Result<AppPreferences, PreferenceError> {
        let mut store = self.load()?;
        mutate(&mut store.preferences);
        let expected = store.preferences;
        self.save(&store)?;

        let saved = self.load()?.preferences;
        if saved == expected {
            Ok(saved)
        } else {
            Err(PreferenceError::StorageWriteFailed)
        }
    }

    fn load(&self) -> Result<PreferencesStore, PreferenceError> {
        let path = self.path()?;
        recover_backup_if_needed(path)?;

        if !path.exists() {
            return Ok(PreferencesStore {
                version: STORE_VERSION,
                preferences: AppPreferences::default(),
            });
        }

        let contents = fs::read(path).map_err(|_| PreferenceError::StorageReadFailed)?;
        let store: PreferencesStore =
            serde_json::from_slice(&contents).map_err(|_| PreferenceError::MalformedData)?;
        if store.version != STORE_VERSION {
            return Err(PreferenceError::MalformedData);
        }
        Ok(store)
    }

    fn save(&self, store: &PreferencesStore) -> Result<(), PreferenceError> {
        let path = self.path()?;
        let parent = path.parent().ok_or(PreferenceError::StorageUnavailable)?;
        fs::create_dir_all(parent).map_err(|_| PreferenceError::StorageWriteFailed)?;

        let contents =
            serde_json::to_vec_pretty(store).map_err(|_| PreferenceError::StorageWriteFailed)?;
        let temporary_path = parent.join(format!(
            ".{}.{}.tmp",
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("preferences"),
            Uuid::new_v4()
        ));

        let result = (|| {
            let mut temporary = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temporary_path)
                .map_err(|_| PreferenceError::StorageWriteFailed)?;
            temporary
                .write_all(&contents)
                .and_then(|_| temporary.sync_all())
                .map_err(|_| PreferenceError::StorageWriteFailed)?;
            replace_with_backup(path, &temporary_path)
        })();

        if result.is_err() {
            let _ = fs::remove_file(&temporary_path);
        }
        result
    }

    fn path(&self) -> Result<&Path, PreferenceError> {
        self.store_path
            .as_deref()
            .ok_or(PreferenceError::StorageUnavailable)
    }
}

fn backup_path(path: &Path) -> PathBuf {
    path.with_extension("json.backup")
}

fn recover_backup_if_needed(path: &Path) -> Result<(), PreferenceError> {
    let backup = backup_path(path);
    if !path.exists() && backup.exists() {
        fs::rename(backup, path).map_err(|_| PreferenceError::StorageReadFailed)?;
    }
    Ok(())
}

fn replace_with_backup(path: &Path, temporary_path: &Path) -> Result<(), PreferenceError> {
    let backup = backup_path(path);
    if backup.exists() {
        fs::remove_file(&backup).map_err(|_| PreferenceError::StorageWriteFailed)?;
    }

    let had_existing = path.exists();
    if had_existing {
        fs::rename(path, &backup).map_err(|_| PreferenceError::StorageWriteFailed)?;
    }

    if fs::rename(temporary_path, path).is_err() {
        if had_existing {
            let _ = fs::rename(&backup, path);
        }
        return Err(PreferenceError::StorageWriteFailed);
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
                "git-identity-manager-preferences-test-{}",
                Uuid::new_v4()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn service_in(directory: &Path) -> PreferencesService {
        PreferencesService::new(Some(directory.join("preferences.v1.json")))
    }

    #[test]
    fn missing_store_returns_defaults_without_writing() {
        let directory = TestDirectory::new();
        let service = service_in(directory.path());

        assert_eq!(service.get().unwrap(), AppPreferences::default());
        assert_eq!(service.get().unwrap().theme, ThemePreference::System);
        assert!(!service.get().unwrap().welcome_dismissed);
        assert!(!directory.path().join("preferences.v1.json").exists());
    }

    #[test]
    fn theme_and_welcome_updates_persist_across_restart() {
        let directory = TestDirectory::new();

        let first = service_in(directory.path());
        assert_eq!(
            first.set_theme(ThemePreference::Light).unwrap().theme,
            ThemePreference::Light
        );
        assert!(first.set_welcome_dismissed(true).unwrap().welcome_dismissed);

        let restarted = service_in(directory.path());
        let preferences = restarted.get().unwrap();
        assert_eq!(preferences.theme, ThemePreference::Light);
        assert!(preferences.welcome_dismissed);
    }

    #[test]
    fn updates_only_touch_the_targeted_field() {
        let directory = TestDirectory::new();
        let service = service_in(directory.path());

        service.set_welcome_dismissed(true).unwrap();
        let preferences = service.set_theme(ThemePreference::Dark).unwrap();

        assert_eq!(preferences.theme, ThemePreference::Dark);
        assert!(preferences.welcome_dismissed);
    }

    #[test]
    fn stored_preferences_contain_only_theme_and_welcome_flag() {
        let directory = TestDirectory::new();
        let service = service_in(directory.path());
        service.set_theme(ThemePreference::Dark).unwrap();

        let raw = fs::read_to_string(directory.path().join("preferences.v1.json")).unwrap();
        let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(value["version"], 1);
        let keys: Vec<&str> = value["preferences"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, ["theme", "welcomeDismissed"]);
    }

    #[test]
    fn wrong_version_is_rejected_as_malformed() {
        let directory = TestDirectory::new();
        let path = directory.path().join("preferences.v1.json");
        fs::write(
            &path,
            r#"{"version":2,"preferences":{"theme":"dark","welcomeDismissed":false}}"#,
        )
        .unwrap();

        assert_eq!(
            service_in(directory.path()).get(),
            Err(PreferenceError::MalformedData)
        );
    }

    #[test]
    fn unknown_theme_value_is_rejected_as_malformed() {
        let directory = TestDirectory::new();
        let path = directory.path().join("preferences.v1.json");
        fs::write(
            &path,
            r#"{"version":1,"preferences":{"theme":"sepia","welcomeDismissed":false}}"#,
        )
        .unwrap();

        assert_eq!(
            service_in(directory.path()).get(),
            Err(PreferenceError::MalformedData)
        );
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let directory = TestDirectory::new();
        let path = directory.path().join("preferences.v1.json");
        fs::write(&path, r#"{"version":1,"preferences":{"theme":"dark"}}"#).unwrap();

        let preferences = service_in(directory.path()).get().unwrap();
        assert_eq!(preferences.theme, ThemePreference::Dark);
        assert!(!preferences.welcome_dismissed);
    }

    #[test]
    fn recovers_from_backup_when_primary_file_is_missing() {
        let directory = TestDirectory::new();
        let service = service_in(directory.path());
        service.set_theme(ThemePreference::Dark).unwrap();

        let path = directory.path().join("preferences.v1.json");
        let backup = directory.path().join("preferences.v1.json.backup");
        fs::rename(&path, &backup).unwrap();

        assert_eq!(service.get().unwrap().theme, ThemePreference::Dark);
        assert!(path.exists());
    }
}
