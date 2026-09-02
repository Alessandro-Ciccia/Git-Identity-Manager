use serde::Serialize;

use crate::process::{ProcessErrorKind, ProcessOutput, ProcessRunner};

const VERSION_ARGUMENTS: &[&str] = &["--version"];

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EnvironmentStatus {
    pub(crate) git: DependencyStatus,
    pub(crate) github_cli: DependencyStatus,
    pub(crate) is_ready: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DependencyStatus {
    pub(crate) dependency: Dependency,
    pub(crate) state: DependencyState,
    pub(crate) version: Option<String>,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum Dependency {
    Git,
    GithubCli,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum DependencyState {
    Available,
    Missing,
    Error,
}

pub(crate) struct EnvironmentService<R> {
    runner: R,
}

impl<R: ProcessRunner> EnvironmentService<R> {
    pub(crate) fn new(runner: R) -> Self {
        Self { runner }
    }

    pub(crate) fn inspect(&self) -> EnvironmentStatus {
        let git = self.inspect_dependency(Dependency::Git);
        let github_cli = self.inspect_dependency(Dependency::GithubCli);
        let is_ready = git.state == DependencyState::Available
            && github_cli.state == DependencyState::Available;

        EnvironmentStatus {
            git,
            github_cli,
            is_ready,
        }
    }

    fn inspect_dependency(&self, dependency: Dependency) -> DependencyStatus {
        let program = dependency.executable();

        match self.runner.run(program, VERSION_ARGUMENTS) {
            Ok(output) if output.success => match parse_version(dependency, &output.stdout) {
                Some(version) => DependencyStatus::available(dependency, version),
                None => DependencyStatus::error(
                    dependency,
                    format!(
                        "{} responded, but its version could not be read.",
                        dependency.display_name()
                    ),
                ),
            },
            Ok(ProcessOutput { exit_code, .. }) => {
                let suffix = exit_code
                    .map(|code| format!(" (exit code {code})"))
                    .unwrap_or_default();
                DependencyStatus::error(
                    dependency,
                    format!(
                        "{} could not be checked{suffix}. Verify the installation and try again.",
                        dependency.display_name()
                    ),
                )
            }
            Err(error) if error.kind == ProcessErrorKind::NotFound => {
                DependencyStatus::missing(dependency)
            }
            Err(error) if error.kind == ProcessErrorKind::PermissionDenied => {
                DependencyStatus::error(
                    dependency,
                    format!(
                        "{} was found but could not be executed because permission was denied.",
                        dependency.display_name()
                    ),
                )
            }
            Err(_) => DependencyStatus::error(
                dependency,
                format!(
                    "{} could not be checked. Verify the installation and try again.",
                    dependency.display_name()
                ),
            ),
        }
    }
}

impl Dependency {
    fn executable(self) -> &'static str {
        match self {
            Self::Git => "git",
            Self::GithubCli => "gh",
        }
    }

    fn display_name(self) -> &'static str {
        match self {
            Self::Git => "Git",
            Self::GithubCli => "GitHub CLI",
        }
    }

    fn version_prefix(self) -> &'static str {
        match self {
            Self::Git => "git version ",
            Self::GithubCli => "gh version ",
        }
    }
}

impl DependencyStatus {
    fn available(dependency: Dependency, version: String) -> Self {
        Self {
            dependency,
            state: DependencyState::Available,
            message: format!("{} is available.", dependency.display_name()),
            version: Some(version),
        }
    }

    fn missing(dependency: Dependency) -> Self {
        Self {
            dependency,
            state: DependencyState::Missing,
            message: format!(
                "{} is not installed or is not available on PATH. Install it, then refresh this check.",
                dependency.display_name()
            ),
            version: None,
        }
    }

    fn error(dependency: Dependency, message: String) -> Self {
        Self {
            dependency,
            state: DependencyState::Error,
            version: None,
            message,
        }
    }
}

fn parse_version(dependency: Dependency, output: &str) -> Option<String> {
    output
        .lines()
        .find(|line| !line.trim().is_empty())?
        .trim()
        .strip_prefix(dependency.version_prefix())?
        .split_whitespace()
        .next()
        .filter(|version| !version.is_empty())
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Mutex};

    use super::*;
    use crate::process::{ProcessError, ProcessErrorKind};

    #[derive(Default)]
    struct FakeProcessRunner {
        responses: HashMap<&'static str, Result<ProcessOutput, ProcessError>>,
        calls: Mutex<Vec<(String, Vec<String>)>>,
    }

    impl FakeProcessRunner {
        fn with_response(
            mut self,
            program: &'static str,
            response: Result<ProcessOutput, ProcessError>,
        ) -> Self {
            self.responses.insert(program, response);
            self
        }

        fn calls(&self) -> Vec<(String, Vec<String>)> {
            self.calls.lock().expect("calls mutex poisoned").clone()
        }
    }

    impl ProcessRunner for &FakeProcessRunner {
        fn run(&self, program: &str, args: &[&str]) -> Result<ProcessOutput, ProcessError> {
            self.calls.lock().expect("calls mutex poisoned").push((
                program.to_owned(),
                args.iter().map(|arg| (*arg).to_owned()).collect(),
            ));

            self.responses
                .get(program)
                .cloned()
                .unwrap_or_else(|| panic!("missing fake response for {program}"))
        }

        fn start(&self, _program: &str, _args: &[&str]) -> Result<(), ProcessError> {
            panic!("environment detection must not start background processes")
        }
    }

    fn successful(stdout: &str) -> Result<ProcessOutput, ProcessError> {
        Ok(ProcessOutput {
            success: true,
            exit_code: Some(0),
            stdout: stdout.to_owned(),
        })
    }

    fn missing() -> Result<ProcessOutput, ProcessError> {
        Err(ProcessError {
            kind: ProcessErrorKind::NotFound,
        })
    }

    #[test]
    fn checks_fixed_commands_and_parses_available_versions() {
        let runner = FakeProcessRunner::default()
            .with_response("git", successful("git version 2.47.0\n"))
            .with_response(
                "gh",
                successful("gh version 2.73.0 (2025-05-19)\nhttps://github.com/cli/cli/releases/tag/v2.73.0\n"),
            );

        let status = EnvironmentService::new(&runner).inspect();

        assert_eq!(status.git.state, DependencyState::Available);
        assert_eq!(status.git.version.as_deref(), Some("2.47.0"));
        assert_eq!(status.github_cli.state, DependencyState::Available);
        assert_eq!(status.github_cli.version.as_deref(), Some("2.73.0"));
        assert!(status.is_ready);
        assert_eq!(
            runner.calls(),
            vec![
                ("git".to_owned(), vec!["--version".to_owned()]),
                ("gh".to_owned(), vec!["--version".to_owned()]),
            ]
        );
    }

    #[test]
    fn reports_git_as_missing_without_skipping_github_cli() {
        let runner = FakeProcessRunner::default()
            .with_response("git", missing())
            .with_response("gh", successful("gh version 2.73.0 (2025-05-19)\n"));

        let status = EnvironmentService::new(&runner).inspect();

        assert_eq!(status.git.state, DependencyState::Missing);
        assert_eq!(status.git.version, None);
        assert_eq!(status.github_cli.state, DependencyState::Available);
        assert!(!status.is_ready);
        assert_eq!(runner.calls().len(), 2);
    }

    #[test]
    fn reports_github_cli_as_missing() {
        let runner = FakeProcessRunner::default()
            .with_response("git", successful("git version 2.47.0\n"))
            .with_response("gh", missing());

        let status = EnvironmentService::new(&runner).inspect();

        assert_eq!(status.git.state, DependencyState::Available);
        assert_eq!(status.github_cli.state, DependencyState::Missing);
        assert!(!status.is_ready);
    }

    #[test]
    fn reports_malformed_successful_output_as_an_error() {
        let runner = FakeProcessRunner::default()
            .with_response("git", successful("unexpected output\n"))
            .with_response("gh", successful("gh version 2.73.0 (2025-05-19)\n"));

        let status = EnvironmentService::new(&runner).inspect();

        assert_eq!(status.git.state, DependencyState::Error);
        assert_eq!(status.git.version, None);
        assert!(!status.git.message.contains("unexpected output"));
        assert!(!status.is_ready);
    }

    #[test]
    fn reports_non_zero_process_exit_as_an_error_without_raw_output() {
        let runner = FakeProcessRunner::default()
            .with_response(
                "git",
                Ok(ProcessOutput {
                    success: false,
                    exit_code: Some(42),
                    stdout: "sensitive diagnostic".to_owned(),
                }),
            )
            .with_response("gh", successful("gh version 2.73.0 (2025-05-19)\n"));

        let status = EnvironmentService::new(&runner).inspect();

        assert_eq!(status.git.state, DependencyState::Error);
        assert!(status.git.message.contains("exit code 42"));
        assert!(!status.git.message.contains("sensitive diagnostic"));
        assert_eq!(status.github_cli.state, DependencyState::Available);
    }

    #[test]
    fn reports_process_launch_failure_as_an_error() {
        let runner = FakeProcessRunner::default()
            .with_response(
                "git",
                Err(ProcessError {
                    kind: ProcessErrorKind::Other,
                }),
            )
            .with_response("gh", successful("gh version 2.73.0 (2025-05-19)\n"));

        let status = EnvironmentService::new(&runner).inspect();

        assert_eq!(status.git.state, DependencyState::Error);
        assert_eq!(status.github_cli.state, DependencyState::Available);
        assert!(!status.is_ready);
    }
}
