use serde::Serialize;

use crate::services::{
    git::GitInspectionError, github_cli::GithubCliError, profiles::ProfileError,
    repositories::RepositoryError,
};

#[derive(Debug, Serialize)]
#[serde(tag = "code", rename_all = "camelCase")]
pub(crate) enum AppError {
    EnvironmentCheckFailed { message: &'static str },
    GithubCliMissing { message: &'static str },
    GithubStatusFailed { message: &'static str },
    GithubStatusMalformed { message: &'static str },
    InvalidGithubHostname { message: &'static str },
    InvalidGithubUsername { message: &'static str },
    GithubAccountNotFound { message: &'static str },
    GithubAccountUnavailable { message: &'static str },
    GithubSwitchFailed { message: &'static str },
    GithubSwitchNotVerified { message: &'static str },
    GithubLoginFailed { message: &'static str },
    GithubOperationFailed { message: &'static str },
    GithubLoginPageOpenFailed { message: &'static str },
    GithubAccountPageOpenFailed { message: &'static str },
    ExternalPageOpenFailed { message: &'static str },
    ProfileStorageUnavailable { message: &'static str },
    ProfileStorageFailed { message: &'static str },
    ProfileDataMalformed { message: &'static str },
    InvalidProfileId { message: &'static str },
    InvalidProfileLabel { message: &'static str },
    InvalidGitName { message: &'static str },
    InvalidGitEmail { message: &'static str },
    InvalidProfileGithubAccount { message: &'static str },
    ProfileNotFound { message: &'static str },
    ProfileOperationFailed { message: &'static str },
    RepositoryStorageUnavailable { message: &'static str },
    RepositoryStorageFailed { message: &'static str },
    RepositoryDataMalformed { message: &'static str },
    InvalidRepositoryId { message: &'static str },
    InvalidRepositoryPath { message: &'static str },
    RepositoryPathMissing { message: &'static str },
    InvalidRepository { message: &'static str },
    GitMissing { message: &'static str },
    GitCommandFailed { message: &'static str },
    GitOutputMalformed { message: &'static str },
    RepositoryNotFound { message: &'static str },
    RepositoryRevealFailed { message: &'static str },
    RepositoryOperationFailed { message: &'static str },
}

impl AppError {
    pub(crate) fn environment_check_failed() -> Self {
        Self::EnvironmentCheckFailed {
            message: "The environment check could not be completed. Please try again.",
        }
    }

    pub(crate) fn github_operation_failed() -> Self {
        Self::GithubOperationFailed {
            message: "The GitHub account operation could not be completed. Please try again.",
        }
    }

    pub(crate) fn github_login_page_open_failed() -> Self {
        Self::GithubLoginPageOpenFailed {
            message: "The GitHub login page could not be opened. Use Open browser to try again.",
        }
    }

    pub(crate) fn github_account_page_open_failed() -> Self {
        Self::GithubAccountPageOpenFailed {
            message: "The GitHub account page could not be opened. Please try again.",
        }
    }

    pub(crate) fn external_page_open_failed() -> Self {
        Self::ExternalPageOpenFailed {
            message: "The GitHub CLI installation page could not be opened. Please try again.",
        }
    }

    pub(crate) fn profile_operation_failed() -> Self {
        Self::ProfileOperationFailed {
            message: "The profile operation could not be completed. Please try again.",
        }
    }

    pub(crate) fn repository_operation_failed() -> Self {
        Self::RepositoryOperationFailed {
            message: "The repository operation could not be completed. Please try again.",
        }
    }

    pub(crate) fn repository_path_missing() -> Self {
        Self::RepositoryPathMissing {
            message: "This repository folder is missing or has moved.",
        }
    }

    pub(crate) fn repository_reveal_failed() -> Self {
        Self::RepositoryRevealFailed {
            message: "The repository folder could not be opened.",
        }
    }
}

impl From<GitInspectionError> for AppError {
    fn from(error: GitInspectionError) -> Self {
        match error {
            GitInspectionError::InvalidPath | GitInspectionError::NotDirectory => {
                Self::InvalidRepositoryPath {
                    message: "Select a valid repository folder.",
                }
            }
            GitInspectionError::PathNotFound => Self::RepositoryPathMissing {
                message: "The selected folder does not exist or is unavailable.",
            },
            GitInspectionError::NotRepository => Self::InvalidRepository {
                message: "The selected folder is not inside a Git working repository.",
            },
            GitInspectionError::GitMissing => Self::GitMissing {
                message: "Git is not installed or is not available on PATH.",
            },
            GitInspectionError::CommandFailed => Self::GitCommandFailed {
                message: "Git could not inspect this repository. Check access and try again.",
            },
            GitInspectionError::MalformedOutput => Self::GitOutputMalformed {
                message: "Git returned repository information that could not be read safely.",
            },
        }
    }
}

impl From<RepositoryError> for AppError {
    fn from(error: RepositoryError) -> Self {
        match error {
            RepositoryError::StorageUnavailable => Self::RepositoryStorageUnavailable {
                message: "Local repository storage is not available on this system.",
            },
            RepositoryError::StorageReadFailed | RepositoryError::StorageWriteFailed => {
                Self::RepositoryStorageFailed {
                    message: "Repositories could not be read or saved. Check application data permissions and try again.",
                }
            }
            RepositoryError::MalformedData => Self::RepositoryDataMalformed {
                message: "Stored repository data could not be read safely.",
            },
            RepositoryError::InvalidId => Self::InvalidRepositoryId {
                message: "The repository identifier is invalid.",
            },
            RepositoryError::InvalidPath => Self::InvalidRepositoryPath {
                message: "The repository path is invalid.",
            },
            RepositoryError::NotFound => Self::RepositoryNotFound {
                message: "That repository is no longer registered. Refresh and try again.",
            },
            RepositoryError::ClockUnavailable => Self::RepositoryStorageFailed {
                message: "The repository could not be saved because the system time is unavailable.",
            },
        }
    }
}

impl From<ProfileError> for AppError {
    fn from(error: ProfileError) -> Self {
        match error {
            ProfileError::StorageUnavailable => Self::ProfileStorageUnavailable {
                message: "Local profile storage is not available on this system.",
            },
            ProfileError::StorageReadFailed | ProfileError::StorageWriteFailed => {
                Self::ProfileStorageFailed {
                    message: "Profiles could not be read or saved. Check application data permissions and try again.",
                }
            }
            ProfileError::MalformedData => Self::ProfileDataMalformed {
                message: "Stored profile data could not be read safely.",
            },
            ProfileError::InvalidId => Self::InvalidProfileId {
                message: "The profile identifier is invalid.",
            },
            ProfileError::InvalidLabel => Self::InvalidProfileLabel {
                message: "Enter a profile label between 1 and 80 characters.",
            },
            ProfileError::InvalidGitName => Self::InvalidGitName {
                message: "Enter a Git name between 1 and 200 characters without control characters.",
            },
            ProfileError::InvalidGitEmail => Self::InvalidGitEmail {
                message: "Enter a valid Git email address.",
            },
            ProfileError::InvalidGithubAccount => Self::InvalidProfileGithubAccount {
                message: "The associated GitHub account is invalid.",
            },
            ProfileError::NotFound => Self::ProfileNotFound {
                message: "That profile no longer exists. Refresh profiles and try again.",
            },
        }
    }
}

impl From<GithubCliError> for AppError {
    fn from(error: GithubCliError) -> Self {
        match error {
            GithubCliError::Missing => Self::GithubCliMissing {
                message: "GitHub CLI is not installed or is not available on PATH.",
            },
            GithubCliError::StatusFailed => Self::GithubStatusFailed {
                message: "GitHub CLI could not provide structured account status. Install or update GitHub CLI, then try again.",
            },
            GithubCliError::MalformedStatus => Self::GithubStatusMalformed {
                message: "GitHub CLI returned account information that could not be read. Update GitHub CLI and try again.",
            },
            GithubCliError::InvalidHostname => Self::InvalidGithubHostname {
                message: "The GitHub hostname is invalid.",
            },
            GithubCliError::InvalidUsername => Self::InvalidGithubUsername {
                message: "The GitHub username is invalid.",
            },
            GithubCliError::AccountNotFound => Self::GithubAccountNotFound {
                message: "That GitHub account is no longer available. Refresh and try again.",
            },
            GithubCliError::AccountUnavailable => Self::GithubAccountUnavailable {
                message: "That GitHub account has an authentication issue and cannot be activated.",
            },
            GithubCliError::SwitchFailed => Self::GithubSwitchFailed {
                message: "GitHub CLI could not switch accounts. Refresh the account status and try again.",
            },
            GithubCliError::SwitchNotVerified => Self::GithubSwitchNotVerified {
                message: "GitHub CLI completed the switch, but the selected account did not become active.",
            },
            GithubCliError::LoginFailed => Self::GithubLoginFailed {
                message: "GitHub browser login did not complete. Try again and finish the flow in your browser.",
            },
        }
    }
}
