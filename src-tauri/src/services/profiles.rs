use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

const STORE_VERSION: u32 = 1;
const LABEL_MAX_CHARS: usize = 80;
const GIT_NAME_MAX_CHARS: usize = 200;
const EMAIL_MAX_CHARS: usize = 254;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GitProfile {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) git_name: String,
    pub(crate) git_email: String,
    pub(crate) github_account: Option<GithubAccountReference>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProfileInput {
    pub(crate) label: String,
    pub(crate) git_name: String,
    pub(crate) git_email: String,
    pub(crate) github_account: Option<GithubAccountReference>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GithubAccountReference {
    pub(crate) hostname: String,
    pub(crate) username: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProfileError {
    StorageUnavailable,
    StorageReadFailed,
    StorageWriteFailed,
    MalformedData,
    InvalidId,
    InvalidLabel,
    InvalidGitName,
    InvalidGitEmail,
    InvalidGithubAccount,
    NotFound,
}

pub(crate) struct ProfilesService {
    store_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProfilesStore {
    version: u32,
    profiles: Vec<GitProfile>,
}

impl ProfilesService {
    pub(crate) fn new(store_path: Option<PathBuf>) -> Self {
        Self { store_path }
    }

    pub(crate) fn list(&self) -> Result<Vec<GitProfile>, ProfileError> {
        Ok(self.load()?.profiles)
    }

    pub(crate) fn create(&self, input: ProfileInput) -> Result<GitProfile, ProfileError> {
        let input = normalize_and_validate_input(input)?;
        let mut store = self.load()?;
        let profile = GitProfile {
            id: Uuid::new_v4().to_string(),
            label: input.label,
            git_name: input.git_name,
            git_email: input.git_email,
            github_account: input.github_account,
        };
        store.profiles.push(profile.clone());
        self.save(&store)?;

        self.load()?
            .profiles
            .into_iter()
            .find(|saved| saved.id == profile.id)
            .ok_or(ProfileError::StorageWriteFailed)
    }

    pub(crate) fn update(&self, id: &str, input: ProfileInput) -> Result<GitProfile, ProfileError> {
        validate_id(id)?;
        let input = normalize_and_validate_input(input)?;
        let mut store = self.load()?;
        let profile = store
            .profiles
            .iter_mut()
            .find(|profile| profile.id == id)
            .ok_or(ProfileError::NotFound)?;

        profile.label = input.label;
        profile.git_name = input.git_name;
        profile.git_email = input.git_email;
        profile.github_account = input.github_account;
        let updated = profile.clone();
        self.save(&store)?;

        self.load()?
            .profiles
            .into_iter()
            .find(|saved| saved.id == updated.id && saved == &updated)
            .ok_or(ProfileError::StorageWriteFailed)
    }

    pub(crate) fn delete(&self, id: &str) -> Result<(), ProfileError> {
        validate_id(id)?;
        let mut store = self.load()?;
        let original_len = store.profiles.len();
        store.profiles.retain(|profile| profile.id != id);
        if store.profiles.len() == original_len {
            return Err(ProfileError::NotFound);
        }

        self.save(&store)?;
        if self.load()?.profiles.iter().any(|profile| profile.id == id) {
            return Err(ProfileError::StorageWriteFailed);
        }

        Ok(())
    }

    fn load(&self) -> Result<ProfilesStore, ProfileError> {
        let path = self.path()?;
        recover_backup_if_needed(path)?;

        if !path.exists() {
            return Ok(ProfilesStore {
                version: STORE_VERSION,
                profiles: Vec::new(),
            });
        }

        let contents = fs::read(path).map_err(|_| ProfileError::StorageReadFailed)?;
        let store: ProfilesStore =
            serde_json::from_slice(&contents).map_err(|_| ProfileError::MalformedData)?;
        validate_store(&store)?;
        Ok(store)
    }

    fn save(&self, store: &ProfilesStore) -> Result<(), ProfileError> {
        let path = self.path()?;
        let parent = path.parent().ok_or(ProfileError::StorageUnavailable)?;
        fs::create_dir_all(parent).map_err(|_| ProfileError::StorageWriteFailed)?;

        let contents =
            serde_json::to_vec_pretty(store).map_err(|_| ProfileError::StorageWriteFailed)?;
        let temporary_path = parent.join(format!(
            ".{}.{}.tmp",
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("profiles"),
            Uuid::new_v4()
        ));

        let result = (|| {
            let mut temporary = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temporary_path)
                .map_err(|_| ProfileError::StorageWriteFailed)?;
            temporary
                .write_all(&contents)
                .and_then(|_| temporary.sync_all())
                .map_err(|_| ProfileError::StorageWriteFailed)?;
            replace_with_backup(path, &temporary_path)
        })();

        if result.is_err() {
            let _ = fs::remove_file(&temporary_path);
        }
        result
    }

    fn path(&self) -> Result<&Path, ProfileError> {
        self.store_path
            .as_deref()
            .ok_or(ProfileError::StorageUnavailable)
    }
}

fn normalize_and_validate_input(mut input: ProfileInput) -> Result<ProfileInput, ProfileError> {
    input.label = input.label.trim().to_owned();
    input.git_name = input.git_name.trim().to_owned();
    input.git_email = input.git_email.trim().to_owned();

    validate_text(&input.label, LABEL_MAX_CHARS).map_err(|_| ProfileError::InvalidLabel)?;
    validate_text(&input.git_name, GIT_NAME_MAX_CHARS).map_err(|_| ProfileError::InvalidGitName)?;
    validate_email(&input.git_email)?;

    if let Some(account) = &mut input.github_account {
        account.hostname = account.hostname.trim().to_ascii_lowercase();
        account.username = account.username.trim().to_owned();
        validate_hostname(&account.hostname).map_err(|_| ProfileError::InvalidGithubAccount)?;
        validate_username(&account.username).map_err(|_| ProfileError::InvalidGithubAccount)?;
    }

    Ok(input)
}

fn validate_store(store: &ProfilesStore) -> Result<(), ProfileError> {
    if store.version != STORE_VERSION {
        return Err(ProfileError::MalformedData);
    }

    let mut ids = std::collections::HashSet::new();
    for profile in &store.profiles {
        validate_id(&profile.id).map_err(|_| ProfileError::MalformedData)?;
        if !ids.insert(profile.id.as_str()) {
            return Err(ProfileError::MalformedData);
        }
        normalize_and_validate_input(ProfileInput {
            label: profile.label.clone(),
            git_name: profile.git_name.clone(),
            git_email: profile.git_email.clone(),
            github_account: profile.github_account.clone(),
        })
        .map_err(|_| ProfileError::MalformedData)?;
    }

    Ok(())
}

fn validate_id(id: &str) -> Result<(), ProfileError> {
    Uuid::parse_str(id)
        .map(|_| ())
        .map_err(|_| ProfileError::InvalidId)
}

fn validate_text(value: &str, max_chars: usize) -> Result<(), ()> {
    let length = value.chars().count();
    if length == 0 || length > max_chars || value.chars().any(char::is_control) {
        Err(())
    } else {
        Ok(())
    }
}

fn validate_email(email: &str) -> Result<(), ProfileError> {
    let valid_length = !email.is_empty() && email.chars().count() <= EMAIL_MAX_CHARS;
    let valid = valid_length
        && !email
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
        && email.split_once('@').is_some_and(|(local, domain)| {
            !local.is_empty()
                && local.len() <= 64
                && !local.starts_with('.')
                && !local.ends_with('.')
                && !local.contains("..")
                && !domain.contains('@')
                && domain.contains('.')
                && validate_hostname(domain).is_ok()
        });

    if valid {
        Ok(())
    } else {
        Err(ProfileError::InvalidGitEmail)
    }
}

fn validate_hostname(hostname: &str) -> Result<(), ()> {
    if hostname.is_empty()
        || hostname.len() > 253
        || !hostname.is_ascii()
        || hostname.starts_with('.')
        || hostname.ends_with('.')
    {
        return Err(());
    }

    if hostname.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            && label
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_alphanumeric)
            && label
                .as_bytes()
                .last()
                .is_some_and(u8::is_ascii_alphanumeric)
    }) {
        Ok(())
    } else {
        Err(())
    }
}

fn validate_username(username: &str) -> Result<(), ()> {
    let bytes = username.as_bytes();
    let valid = !username.is_empty()
        && username.len() <= 100
        && username.is_ascii()
        && bytes.first().is_some_and(u8::is_ascii_alphanumeric)
        && bytes.last().is_some_and(u8::is_ascii_alphanumeric)
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'-');

    if valid {
        Ok(())
    } else {
        Err(())
    }
}

fn backup_path(path: &Path) -> PathBuf {
    path.with_extension("json.backup")
}

fn recover_backup_if_needed(path: &Path) -> Result<(), ProfileError> {
    let backup = backup_path(path);
    if !path.exists() && backup.exists() {
        fs::rename(backup, path).map_err(|_| ProfileError::StorageReadFailed)?;
    }
    Ok(())
}

fn replace_with_backup(path: &Path, temporary_path: &Path) -> Result<(), ProfileError> {
    let backup = backup_path(path);
    if backup.exists() {
        fs::remove_file(&backup).map_err(|_| ProfileError::StorageWriteFailed)?;
    }

    let had_existing = path.exists();
    if had_existing {
        fs::rename(path, &backup).map_err(|_| ProfileError::StorageWriteFailed)?;
    }

    if fs::rename(temporary_path, path).is_err() {
        if had_existing {
            let _ = fs::rename(&backup, path);
        }
        return Err(ProfileError::StorageWriteFailed);
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
                "git-identity-manager-profiles-test-{}",
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

    fn input(label: &str) -> ProfileInput {
        ProfileInput {
            label: label.to_owned(),
            git_name: "Octo Cat".to_owned(),
            git_email: "octo@example.com".to_owned(),
            github_account: Some(GithubAccountReference {
                hostname: "GitHub.COM".to_owned(),
                username: "octocat".to_owned(),
            }),
        }
    }

    fn service_in(directory: &Path) -> ProfilesService {
        ProfilesService::new(Some(directory.join("profiles.v1.json")))
    }

    #[test]
    fn missing_store_lists_no_profiles() {
        let directory = TestDirectory::new();
        let service = service_in(directory.path());

        assert_eq!(service.list().unwrap(), Vec::<GitProfile>::new());
        assert!(!directory.path().join("profiles.v1.json").exists());
    }

    #[test]
    fn creates_normalized_secret_free_profiles_that_survive_restart() {
        let directory = TestDirectory::new();
        let service = service_in(directory.path());
        let mut profile_input = input("  Personal  ");
        profile_input.git_name = "  Octo Cat  ".to_owned();
        profile_input.git_email = "  octo@example.com  ".to_owned();

        let created = service.create(profile_input).unwrap();
        assert!(Uuid::parse_str(&created.id).is_ok());
        assert_eq!(created.label, "Personal");
        assert_eq!(created.git_name, "Octo Cat");
        assert_eq!(created.git_email, "octo@example.com");
        assert_eq!(
            created.github_account.as_ref().unwrap().hostname,
            "github.com"
        );

        let restarted = service_in(directory.path());
        assert_eq!(restarted.list().unwrap(), vec![created]);

        let stored = fs::read_to_string(directory.path().join("profiles.v1.json")).unwrap();
        assert!(stored.contains("\"version\": 1"));
        for prohibited in ["token", "password", "privateKey", "credential"] {
            assert!(!stored.contains(prohibited));
        }
    }

    #[test]
    fn updates_without_changing_id_and_persists_the_result() {
        let directory = TestDirectory::new();
        let service = service_in(directory.path());
        let created = service.create(input("Personal")).unwrap();
        let mut update = input("Work");
        update.git_email = "octo@company.example".to_owned();
        update.github_account = None;

        let updated = service.update(&created.id, update).unwrap();

        assert_eq!(updated.id, created.id);
        assert_eq!(updated.label, "Work");
        assert_eq!(updated.github_account, None);
        assert_eq!(service_in(directory.path()).list().unwrap(), vec![updated]);
    }

    #[test]
    fn deletes_a_profile_and_persists_its_removal() {
        let directory = TestDirectory::new();
        let service = service_in(directory.path());
        let created = service.create(input("Personal")).unwrap();

        service.delete(&created.id).unwrap();

        assert!(service_in(directory.path()).list().unwrap().is_empty());
        assert_eq!(service.delete(&created.id), Err(ProfileError::NotFound));
    }

    #[test]
    fn rejects_invalid_ids_and_unknown_profiles() {
        let directory = TestDirectory::new();
        let service = service_in(directory.path());

        assert_eq!(
            service.update("not-an-id", input("Work")),
            Err(ProfileError::InvalidId)
        );
        assert_eq!(service.delete("not-an-id"), Err(ProfileError::InvalidId));
        assert_eq!(
            service.update(&Uuid::new_v4().to_string(), input("Work")),
            Err(ProfileError::NotFound)
        );
    }

    #[test]
    fn rejects_invalid_profile_fields() {
        let directory = TestDirectory::new();
        let service = service_in(directory.path());

        let mut invalid = input("   ");
        assert_eq!(
            service.create(invalid.clone()),
            Err(ProfileError::InvalidLabel)
        );

        invalid = input("Profile");
        invalid.git_name = "name\nother".to_owned();
        assert_eq!(
            service.create(invalid.clone()),
            Err(ProfileError::InvalidGitName)
        );

        invalid = input("Profile");
        invalid.git_email = "not-an-email".to_owned();
        assert_eq!(
            service.create(invalid.clone()),
            Err(ProfileError::InvalidGitEmail)
        );

        invalid = input("Profile");
        invalid.github_account.as_mut().unwrap().hostname = "https://github.com".to_owned();
        assert_eq!(
            service.create(invalid.clone()),
            Err(ProfileError::InvalidGithubAccount)
        );

        invalid = input("Profile");
        invalid.github_account.as_mut().unwrap().username = "-octocat".to_owned();
        assert_eq!(
            service.create(invalid),
            Err(ProfileError::InvalidGithubAccount)
        );
    }

    #[test]
    fn rejects_malformed_duplicate_or_future_store_data() {
        let directory = TestDirectory::new();
        let path = directory.path().join("profiles.v1.json");
        fs::write(&path, "not json").unwrap();
        assert_eq!(
            service_in(directory.path()).list(),
            Err(ProfileError::MalformedData)
        );

        fs::write(&path, r#"{"version":2,"profiles":[]}"#).unwrap();
        assert_eq!(
            service_in(directory.path()).list(),
            Err(ProfileError::MalformedData)
        );

        let id = Uuid::new_v4();
        fs::write(
            &path,
            format!(
                r#"{{"version":1,"profiles":[{{"id":"{id}","label":"One","gitName":"Octo","gitEmail":"octo@example.com","githubAccount":null}},{{"id":"{id}","label":"Two","gitName":"Octo","gitEmail":"octo@example.com","githubAccount":null}}]}}"#
            ),
        )
        .unwrap();
        assert_eq!(
            service_in(directory.path()).list(),
            Err(ProfileError::MalformedData)
        );
    }

    #[test]
    fn reports_unavailable_and_unwritable_storage_safely() {
        let unavailable = ProfilesService::new(None);
        assert_eq!(unavailable.list(), Err(ProfileError::StorageUnavailable));

        let directory = TestDirectory::new();
        let store_path = directory.path().join("directory-instead-of-file");
        fs::create_dir(&store_path).unwrap();
        let service = ProfilesService::new(Some(store_path));
        assert_eq!(service.list(), Err(ProfileError::StorageReadFailed));
    }
}
