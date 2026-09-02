use serde::Serialize;

use crate::services::github_cli::GithubCliError;

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
