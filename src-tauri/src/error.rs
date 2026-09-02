use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "code", rename_all = "camelCase")]
pub(crate) enum AppError {
    EnvironmentCheckFailed { message: &'static str },
}

impl AppError {
    pub(crate) fn environment_check_failed() -> Self {
        Self::EnvironmentCheckFailed {
            message: "The environment check could not be completed. Please try again.",
        }
    }
}
