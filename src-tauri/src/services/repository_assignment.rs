use serde::Serialize;

use crate::process::ProcessRunner;

use super::{
    git::{
        GitConfigValue, GitIdentityApplyError, GitInspectionError, GitService, RepositoryInspection,
    },
    profiles::{GitProfile, ProfileError, ProfilesService},
    repositories::{RepositoriesService, RepositoryError, RepositoryRegistration},
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RepositoryProfilePreview {
    pub(crate) repository_id: String,
    pub(crate) path: String,
    pub(crate) profile: GitProfile,
    pub(crate) changes: Vec<RepositoryIdentityChange>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RepositoryIdentityChange {
    pub(crate) key: RepositoryIdentityKey,
    pub(crate) current: GitConfigValue,
    pub(crate) desired: String,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum RepositoryIdentityKey {
    UserName,
    UserEmail,
}

pub(crate) struct RepositoryProfileApplyResult {
    pub(crate) registration: RepositoryRegistration,
    pub(crate) inspection: RepositoryInspection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RepositoryAssignmentError {
    Repository(RepositoryError),
    Profile(ProfileError),
    GitInspection(GitInspectionError),
    GitApply(GitIdentityApplyError),
    NotAssigned,
}

pub(crate) struct RepositoryAssignmentService<'a, R> {
    repositories: &'a RepositoriesService,
    profiles: &'a ProfilesService,
    git: GitService<R>,
}

impl<'a, R: ProcessRunner> RepositoryAssignmentService<'a, R> {
    pub(crate) fn new(
        repositories: &'a RepositoriesService,
        profiles: &'a ProfilesService,
        runner: R,
    ) -> Self {
        Self {
            repositories,
            profiles,
            git: GitService::new(runner),
        }
    }

    pub(crate) fn assign(
        &self,
        repository_id: &str,
        profile_id: &str,
    ) -> Result<RepositoryRegistration, RepositoryAssignmentError> {
        self.profiles
            .find(profile_id)
            .map_err(RepositoryAssignmentError::Profile)?;
        self.repositories
            .assign_profile(repository_id, profile_id)
            .map_err(RepositoryAssignmentError::Repository)
    }

    pub(crate) fn unassign(
        &self,
        repository_id: &str,
    ) -> Result<RepositoryRegistration, RepositoryAssignmentError> {
        self.repositories
            .remove_profile(repository_id)
            .map_err(RepositoryAssignmentError::Repository)
    }

    pub(crate) fn preview(
        &self,
        repository_id: &str,
    ) -> Result<RepositoryProfilePreview, RepositoryAssignmentError> {
        let (registration, profile) = self.assigned(repository_id)?;
        let inspection = self
            .git
            .inspect(&registration.path)
            .map_err(RepositoryAssignmentError::GitInspection)?;

        Ok(RepositoryProfilePreview {
            repository_id: registration.id,
            path: inspection.path,
            changes: vec![
                RepositoryIdentityChange {
                    key: RepositoryIdentityKey::UserName,
                    current: inspection.identity.name,
                    desired: profile.git_name.clone(),
                },
                RepositoryIdentityChange {
                    key: RepositoryIdentityKey::UserEmail,
                    current: inspection.identity.email,
                    desired: profile.git_email.clone(),
                },
            ],
            profile,
        })
    }

    pub(crate) fn apply(
        &self,
        repository_id: &str,
    ) -> Result<RepositoryProfileApplyResult, RepositoryAssignmentError> {
        let (registration, profile) = self.assigned(repository_id)?;
        let inspection = self
            .git
            .apply_local_identity(&registration.path, &profile.git_name, &profile.git_email)
            .map_err(RepositoryAssignmentError::GitApply)?;

        Ok(RepositoryProfileApplyResult {
            registration,
            inspection,
        })
    }

    fn assigned(
        &self,
        repository_id: &str,
    ) -> Result<(RepositoryRegistration, GitProfile), RepositoryAssignmentError> {
        let registration = self
            .repositories
            .find(repository_id)
            .map_err(RepositoryAssignmentError::Repository)?;
        let profile_id = registration
            .profile_id
            .as_deref()
            .ok_or(RepositoryAssignmentError::NotAssigned)?;
        let profile = self
            .profiles
            .find(profile_id)
            .map_err(RepositoryAssignmentError::Profile)?;
        Ok((registration, profile))
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use uuid::Uuid;

    use super::*;
    use crate::{
        process::SystemProcessRunner,
        services::profiles::{GithubAccountReference, ProfileInput},
    };

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "git-identity-manager-assignment-test-{}",
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

    fn run_git(directory: &std::path::Path, arguments: &[&str]) {
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(directory)
            .args(arguments)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .output()
            .unwrap();
        assert!(output.status.success());
    }

    fn profile_input() -> ProfileInput {
        ProfileInput {
            label: "Work".to_owned(),
            git_name: "Octo Cat".to_owned(),
            git_email: "octo@company.example".to_owned(),
            github_account: Some(GithubAccountReference {
                hostname: "github.com".to_owned(),
                username: "octo-work".to_owned(),
            }),
        }
    }

    #[test]
    fn assignment_validates_profiles_and_persists_without_running_git() {
        let directory = TestDirectory::new();
        let repositories = RepositoriesService::new(Some(directory.0.join("repositories.v1.json")));
        let profiles = ProfilesService::new(Some(directory.0.join("profiles.v1.json")));
        let repository = repositories.register("/work/project").unwrap();
        let profile = profiles.create(profile_input()).unwrap();
        let service =
            RepositoryAssignmentService::new(&repositories, &profiles, SystemProcessRunner);

        let assigned = service.assign(&repository.id, &profile.id).unwrap();
        assert_eq!(assigned.profile_id.as_deref(), Some(profile.id.as_str()));
        assert_eq!(
            RepositoriesService::new(Some(directory.0.join("repositories.v1.json")))
                .find(&repository.id)
                .unwrap()
                .profile_id,
            Some(profile.id.clone())
        );

        let unassigned = service.unassign(&repository.id).unwrap();
        assert_eq!(unassigned.profile_id, None);
        assert_eq!(
            service.assign(&repository.id, &Uuid::new_v4().to_string()),
            Err(RepositoryAssignmentError::Profile(ProfileError::NotFound))
        );
    }

    #[test]
    fn preview_uses_stored_data_and_does_not_modify_git_configuration() {
        let directory = TestDirectory::new();
        let repository_path = directory.0.join("repository");
        fs::create_dir(&repository_path).unwrap();
        run_git(&repository_path, &["init", "--quiet"]);
        run_git(
            &repository_path,
            &["config", "--local", "user.name", "Current Octo"],
        );
        run_git(
            &repository_path,
            &["config", "--local", "user.email", "current@example.com"],
        );
        let config_path = repository_path.join(".git/config");
        let config_before = fs::read(&config_path).unwrap();
        let repositories = RepositoriesService::new(Some(directory.0.join("repositories.v1.json")));
        let profiles = ProfilesService::new(Some(directory.0.join("profiles.v1.json")));
        let repository = repositories
            .register(&repository_path.to_string_lossy())
            .unwrap();
        let profile = profiles.create(profile_input()).unwrap();
        let service =
            RepositoryAssignmentService::new(&repositories, &profiles, SystemProcessRunner);
        service.assign(&repository.id, &profile.id).unwrap();

        let preview = service.preview(&repository.id).unwrap();

        assert_eq!(preview.repository_id, repository.id);
        assert_eq!(preview.profile, profile);
        assert_eq!(preview.changes.len(), 2);
        assert_eq!(
            preview.changes[0].current.value.as_deref(),
            Some("Current Octo")
        );
        assert_eq!(preview.changes[0].desired, "Octo Cat");
        assert_eq!(fs::read(config_path).unwrap(), config_before);
    }

    #[test]
    fn stale_and_absent_assignments_cannot_be_previewed_or_applied() {
        let directory = TestDirectory::new();
        let repositories = RepositoriesService::new(Some(directory.0.join("repositories.v1.json")));
        let profiles = ProfilesService::new(Some(directory.0.join("profiles.v1.json")));
        let repository = repositories.register("/work/project").unwrap();
        let service =
            RepositoryAssignmentService::new(&repositories, &profiles, SystemProcessRunner);

        assert_eq!(
            service.preview(&repository.id),
            Err(RepositoryAssignmentError::NotAssigned)
        );

        let stale_id = Uuid::new_v4().to_string();
        repositories
            .assign_profile(&repository.id, &stale_id)
            .unwrap();
        assert_eq!(
            service.apply(&repository.id).map(|_| ()),
            Err(RepositoryAssignmentError::Profile(ProfileError::NotFound))
        );
    }
}
