use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::process::{ProcessErrorKind, ProcessRunner};

const STATUS_ARGUMENTS: &[&str] = &["auth", "status", "--json", "hosts"];

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GithubAccountsStatus {
    pub(crate) accounts: Vec<GithubAccount>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GithubAccount {
    pub(crate) hostname: String,
    pub(crate) username: String,
    pub(crate) email: Option<String>,
    pub(crate) active: bool,
    pub(crate) state: GithubAuthenticationState,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum GithubAuthenticationState {
    Success,
    Error,
    Timeout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GithubCliError {
    Missing,
    StatusFailed,
    MalformedStatus,
    InvalidHostname,
    InvalidUsername,
    AccountNotFound,
    AccountUnavailable,
    SwitchFailed,
    SwitchNotVerified,
    LoginFailed,
}

pub(crate) struct GithubCliService<R> {
    runner: R,
}

impl<R: ProcessRunner> GithubCliService<R> {
    pub(crate) fn new(runner: R) -> Self {
        Self { runner }
    }

    pub(crate) fn list_accounts(&self) -> Result<GithubAccountsStatus, GithubCliError> {
        let output = self
            .runner
            .run("gh", STATUS_ARGUMENTS)
            .map_err(|error| match error.kind {
                ProcessErrorKind::NotFound => GithubCliError::Missing,
                ProcessErrorKind::PermissionDenied | ProcessErrorKind::Other => {
                    GithubCliError::StatusFailed
                }
            })?;

        if !output.success {
            return Err(GithubCliError::StatusFailed);
        }

        parse_accounts(&output.stdout)
    }

    pub(crate) fn list_accounts_with_public_emails(
        &self,
    ) -> Result<GithubAccountsStatus, GithubCliError> {
        let mut status = self.list_accounts()?;
        self.populate_public_emails(&mut status);
        Ok(status)
    }

    pub(crate) fn account_profile_url(
        &self,
        hostname: &str,
        username: &str,
    ) -> Result<String, GithubCliError> {
        validate_hostname(hostname)?;
        validate_username(username)?;

        let status = self.list_accounts()?;
        let account = status
            .accounts
            .iter()
            .find(|account| {
                account.hostname.eq_ignore_ascii_case(hostname)
                    && account.username.eq_ignore_ascii_case(username)
            })
            .ok_or(GithubCliError::AccountNotFound)?;

        Ok(format!("https://{}/{}", account.hostname, account.username))
    }

    pub(crate) fn switch_account_with_public_emails(
        &self,
        hostname: &str,
        username: &str,
    ) -> Result<GithubAccountsStatus, GithubCliError> {
        let mut status = self.switch_account(hostname, username)?;
        self.populate_public_emails(&mut status);
        Ok(status)
    }

    pub(crate) fn switch_account(
        &self,
        hostname: &str,
        username: &str,
    ) -> Result<GithubAccountsStatus, GithubCliError> {
        validate_hostname(hostname)?;
        validate_username(username)?;

        let current = self.list_accounts()?;
        let target = current
            .accounts
            .iter()
            .find(|account| {
                account.hostname.eq_ignore_ascii_case(hostname)
                    && account.username.eq_ignore_ascii_case(username)
            })
            .ok_or(GithubCliError::AccountNotFound)?;

        if target.state != GithubAuthenticationState::Success {
            return Err(GithubCliError::AccountUnavailable);
        }

        if target.active {
            return Ok(current);
        }

        let canonical_hostname = target.hostname.clone();
        let canonical_username = target.username.clone();
        let arguments = [
            "auth",
            "switch",
            "--hostname",
            canonical_hostname.as_str(),
            "--user",
            canonical_username.as_str(),
        ];
        let output = self
            .runner
            .run("gh", &arguments)
            .map_err(|error| match error.kind {
                ProcessErrorKind::NotFound => GithubCliError::Missing,
                ProcessErrorKind::PermissionDenied | ProcessErrorKind::Other => {
                    GithubCliError::SwitchFailed
                }
            })?;

        if !output.success {
            return Err(GithubCliError::SwitchFailed);
        }

        let refreshed = self.list_accounts()?;
        let verified = refreshed.accounts.iter().any(|account| {
            account.hostname.eq_ignore_ascii_case(&canonical_hostname)
                && account.username.eq_ignore_ascii_case(&canonical_username)
                && account.active
                && account.state == GithubAuthenticationState::Success
        });

        if !verified {
            return Err(GithubCliError::SwitchNotVerified);
        }

        Ok(refreshed)
    }

    pub(crate) fn launch_login(
        &self,
        hostname: &str,
    ) -> Result<GithubAccountsStatus, GithubCliError> {
        validate_hostname(hostname)?;
        let current = self.list_accounts()?;

        let arguments = [
            "auth",
            "login",
            "--hostname",
            hostname,
            "--web",
            "--clipboard",
        ];
        self.runner
            .start("gh", &arguments)
            .map_err(|error| match error.kind {
                ProcessErrorKind::NotFound => GithubCliError::Missing,
                ProcessErrorKind::PermissionDenied | ProcessErrorKind::Other => {
                    GithubCliError::LoginFailed
                }
            })?;

        Ok(current)
    }

    fn populate_public_emails(&self, status: &mut GithubAccountsStatus) {
        for account in &mut status.accounts {
            let endpoint = format!("users/{}", account.username);
            let arguments = [
                "api",
                "--hostname",
                account.hostname.as_str(),
                endpoint.as_str(),
            ];
            let Ok(output) = self.runner.run("gh", &arguments) else {
                continue;
            };
            if !output.success {
                continue;
            }

            let Ok(profile) = serde_json::from_str::<GithubPublicProfile>(&output.stdout) else {
                continue;
            };
            account.email = profile.email.filter(|email| validate_email(email));
        }
    }
}

#[derive(Debug, Deserialize)]
struct GithubAuthStatusOutput {
    hosts: HashMap<String, Vec<GithubAuthEntry>>,
}

#[derive(Debug, Deserialize)]
struct GithubAuthEntry {
    state: GithubAuthenticationState,
    active: bool,
    host: String,
    login: String,
}

#[derive(Debug, Deserialize)]
struct GithubPublicProfile {
    email: Option<String>,
}

fn parse_accounts(output: &str) -> Result<GithubAccountsStatus, GithubCliError> {
    let status: GithubAuthStatusOutput =
        serde_json::from_str(output).map_err(|_| GithubCliError::MalformedStatus)?;
    let mut accounts = Vec::new();
    let mut identities = HashSet::new();
    let mut active_hosts = HashSet::new();

    for (hostname, entries) in status.hosts {
        validate_hostname(&hostname).map_err(|_| GithubCliError::MalformedStatus)?;
        let normalized_hostname = hostname.to_ascii_lowercase();

        for entry in entries {
            validate_hostname(&entry.host).map_err(|_| GithubCliError::MalformedStatus)?;
            validate_username(&entry.login).map_err(|_| GithubCliError::MalformedStatus)?;

            if !entry.host.eq_ignore_ascii_case(&hostname) {
                return Err(GithubCliError::MalformedStatus);
            }

            let identity = (
                normalized_hostname.clone(),
                entry.login.to_ascii_lowercase(),
            );
            if !identities.insert(identity) {
                return Err(GithubCliError::MalformedStatus);
            }

            if entry.active && !active_hosts.insert(normalized_hostname.clone()) {
                return Err(GithubCliError::MalformedStatus);
            }

            accounts.push(GithubAccount {
                hostname: normalized_hostname.clone(),
                username: entry.login,
                email: None,
                active: entry.active,
                state: entry.state,
            });
        }
    }

    accounts.sort_by(|left, right| {
        left.hostname.cmp(&right.hostname).then_with(|| {
            left.username
                .to_ascii_lowercase()
                .cmp(&right.username.to_ascii_lowercase())
        })
    });

    Ok(GithubAccountsStatus { accounts })
}

fn validate_hostname(hostname: &str) -> Result<(), GithubCliError> {
    if hostname.is_empty()
        || hostname.len() > 253
        || !hostname.is_ascii()
        || hostname.starts_with('.')
        || hostname.ends_with('.')
    {
        return Err(GithubCliError::InvalidHostname);
    }

    let valid = hostname.split('.').all(|label| {
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
    });

    if valid {
        Ok(())
    } else {
        Err(GithubCliError::InvalidHostname)
    }
}

fn validate_email(email: &str) -> bool {
    !email.is_empty()
        && email.len() <= 254
        && !email.chars().any(char::is_control)
        && email.split_once('@').is_some_and(|(local, domain)| {
            !local.is_empty() && !domain.is_empty() && domain.contains('.')
        })
}

fn validate_username(username: &str) -> Result<(), GithubCliError> {
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
        Err(GithubCliError::InvalidUsername)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, sync::Mutex};

    use super::*;
    use crate::process::{ProcessError, ProcessOutput};

    #[derive(Default)]
    struct FakeProcessRunner {
        responses: Mutex<VecDeque<Result<ProcessOutput, ProcessError>>>,
        calls: Mutex<Vec<(String, Vec<String>)>>,
        start_response: Mutex<Option<Result<(), ProcessError>>>,
        started_calls: Mutex<Vec<(String, Vec<String>)>>,
    }

    impl FakeProcessRunner {
        fn new(responses: Vec<Result<ProcessOutput, ProcessError>>) -> Self {
            Self {
                responses: Mutex::new(responses.into()),
                calls: Mutex::new(Vec::new()),
                start_response: Mutex::new(Some(Ok(()))),
                started_calls: Mutex::new(Vec::new()),
            }
        }

        fn with_start_error(self, error: ProcessError) -> Self {
            *self
                .start_response
                .lock()
                .expect("start response mutex poisoned") = Some(Err(error));
            self
        }

        fn calls(&self) -> Vec<(String, Vec<String>)> {
            self.calls.lock().expect("calls mutex poisoned").clone()
        }

        fn started_calls(&self) -> Vec<(String, Vec<String>)> {
            self.started_calls
                .lock()
                .expect("started calls mutex poisoned")
                .clone()
        }
    }

    impl ProcessRunner for &FakeProcessRunner {
        fn run(&self, program: &str, args: &[&str]) -> Result<ProcessOutput, ProcessError> {
            self.calls.lock().expect("calls mutex poisoned").push((
                program.to_owned(),
                args.iter().map(|argument| (*argument).to_owned()).collect(),
            ));
            self.responses
                .lock()
                .expect("responses mutex poisoned")
                .pop_front()
                .expect("missing fake response")
        }

        fn start(&self, program: &str, args: &[&str]) -> Result<(), ProcessError> {
            self.started_calls
                .lock()
                .expect("started calls mutex poisoned")
                .push((
                    program.to_owned(),
                    args.iter().map(|argument| (*argument).to_owned()).collect(),
                ));
            self.start_response
                .lock()
                .expect("start response mutex poisoned")
                .take()
                .expect("missing fake start response")
        }
    }

    fn successful(stdout: &str) -> Result<ProcessOutput, ProcessError> {
        Ok(ProcessOutput {
            success: true,
            exit_code: Some(0),
            stdout: stdout.to_owned(),
        })
    }

    fn failed(stdout: &str) -> Result<ProcessOutput, ProcessError> {
        Ok(ProcessOutput {
            success: false,
            exit_code: Some(1),
            stdout: stdout.to_owned(),
        })
    }

    fn status_json(active_user: &str) -> String {
        format!(
            r#"{{"hosts":{{"github.com":[{{"state":"success","active":{},"host":"github.com","login":"personal"}},{{"state":"success","active":{},"host":"github.com","login":"work"}}]}}}}"#,
            active_user == "personal",
            active_user == "work"
        )
    }

    #[test]
    fn lists_multiple_hosts_and_accounts_without_credential_fields() {
        let runner = FakeProcessRunner::new(vec![successful(
            r#"{
                "hosts": {
                    "github.example.com": [
                        {"state":"timeout","active":true,"host":"github.example.com","login":"octo-enterprise","token":"secret","tokenSource":"keyring","scopes":"repo"}
                    ],
                    "github.com": [
                        {"state":"success","active":false,"host":"github.com","login":"work"},
                        {"state":"error","active":true,"host":"github.com","login":"personal","error":"sensitive diagnostic"}
                    ]
                }
            }"#,
        )]);

        let status = GithubCliService::new(&runner).list_accounts().unwrap();

        assert_eq!(status.accounts.len(), 3);
        assert_eq!(status.accounts[0].hostname, "github.com");
        assert_eq!(status.accounts[0].username, "personal");
        assert!(status.accounts[0].active);
        assert_eq!(status.accounts[0].state, GithubAuthenticationState::Error);
        assert_eq!(status.accounts[2].hostname, "github.example.com");
        let serialized = serde_json::to_string(&status).unwrap();
        assert!(!serialized.contains("secret"));
        assert!(!serialized.contains("tokenSource"));
        assert!(!serialized.contains("scopes"));
        assert_eq!(
            runner.calls(),
            vec![(
                "gh".to_owned(),
                vec!["auth", "status", "--json", "hosts"]
                    .into_iter()
                    .map(str::to_owned)
                    .collect(),
            )]
        );
    }

    #[test]
    fn enriches_accounts_with_only_valid_public_emails() {
        let runner = FakeProcessRunner::new(vec![
            successful(&status_json("personal")),
            successful(r#"{"email":"personal@example.com"}"#),
            successful(r#"{"email":null}"#),
        ]);

        let status = GithubCliService::new(&runner)
            .list_accounts_with_public_emails()
            .unwrap();

        assert_eq!(
            status.accounts[0].email.as_deref(),
            Some("personal@example.com")
        );
        assert_eq!(status.accounts[1].email, None);
        assert_eq!(
            runner.calls()[1],
            (
                "gh".to_owned(),
                vec!["api", "--hostname", "github.com", "users/personal"]
                    .into_iter()
                    .map(str::to_owned)
                    .collect(),
            )
        );
        assert_eq!(
            runner.calls()[2],
            (
                "gh".to_owned(),
                vec!["api", "--hostname", "github.com", "users/work"]
                    .into_iter()
                    .map(str::to_owned)
                    .collect(),
            )
        );
    }

    #[test]
    fn ignores_failed_or_malformed_public_email_lookups() {
        let runner = FakeProcessRunner::new(vec![
            successful(&status_json("personal")),
            failed("private diagnostic"),
            successful(r#"{"email":"not-an-email"}"#),
        ]);

        let status = GithubCliService::new(&runner)
            .list_accounts_with_public_emails()
            .unwrap();

        assert!(status
            .accounts
            .iter()
            .all(|account| account.email.is_none()));
    }

    #[test]
    fn supports_an_empty_account_list() {
        let runner = FakeProcessRunner::new(vec![successful(r#"{"hosts":{}}"#)]);

        let status = GithubCliService::new(&runner).list_accounts().unwrap();

        assert!(status.accounts.is_empty());
    }

    #[test]
    fn rejects_malformed_or_inconsistent_status_output() {
        let malformed = FakeProcessRunner::new(vec![successful("not json")]);
        assert_eq!(
            GithubCliService::new(&malformed).list_accounts(),
            Err(GithubCliError::MalformedStatus)
        );

        let inconsistent = FakeProcessRunner::new(vec![successful(
            r#"{"hosts":{"github.com":[{"state":"success","active":true,"host":"evil.example","login":"octocat"}]}}"#,
        )]);
        assert_eq!(
            GithubCliService::new(&inconsistent).list_accounts(),
            Err(GithubCliError::MalformedStatus)
        );
    }

    #[test]
    fn reports_missing_and_failed_github_cli_without_exposing_output() {
        let missing_runner = FakeProcessRunner::new(vec![Err(ProcessError {
            kind: ProcessErrorKind::NotFound,
        })]);
        assert_eq!(
            GithubCliService::new(&missing_runner).list_accounts(),
            Err(GithubCliError::Missing)
        );

        let failed_runner = FakeProcessRunner::new(vec![failed("unknown flag plus secret")]);
        assert_eq!(
            GithubCliService::new(&failed_runner).list_accounts(),
            Err(GithubCliError::StatusFailed)
        );
    }

    #[test]
    fn builds_a_profile_url_only_for_a_discovered_account() {
        let runner = FakeProcessRunner::new(vec![successful(&status_json("personal"))]);

        let url = GithubCliService::new(&runner)
            .account_profile_url("GITHUB.COM", "WORK")
            .unwrap();

        assert_eq!(url, "https://github.com/work");
        assert_eq!(runner.calls().len(), 1);
    }

    #[test]
    fn rejects_profile_urls_for_unknown_accounts() {
        let runner = FakeProcessRunner::new(vec![successful(&status_json("personal"))]);

        assert_eq!(
            GithubCliService::new(&runner).account_profile_url("github.com", "unknown"),
            Err(GithubCliError::AccountNotFound)
        );
    }

    #[test]
    fn switches_with_canonical_fixed_arguments_and_verifies_the_result() {
        let runner = FakeProcessRunner::new(vec![
            successful(&status_json("personal")),
            successful("switched"),
            successful(&status_json("work")),
        ]);

        let status = GithubCliService::new(&runner)
            .switch_account("GITHUB.COM", "WORK")
            .unwrap();

        assert!(status
            .accounts
            .iter()
            .any(|account| account.username == "work" && account.active));
        assert_eq!(
            runner.calls()[1],
            (
                "gh".to_owned(),
                vec![
                    "auth",
                    "switch",
                    "--hostname",
                    "github.com",
                    "--user",
                    "work",
                ]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            )
        );
    }

    #[test]
    fn rejects_invalid_unknown_and_unhealthy_switch_targets() {
        let invalid_runner = FakeProcessRunner::default();
        assert_eq!(
            GithubCliService::new(&invalid_runner).switch_account("https://github.com", "work"),
            Err(GithubCliError::InvalidHostname)
        );
        assert!(invalid_runner.calls().is_empty());

        let unknown_runner = FakeProcessRunner::new(vec![successful(&status_json("personal"))]);
        assert_eq!(
            GithubCliService::new(&unknown_runner).switch_account("github.com", "unknown"),
            Err(GithubCliError::AccountNotFound)
        );
        assert_eq!(unknown_runner.calls().len(), 1);

        let unhealthy_runner = FakeProcessRunner::new(vec![successful(
            r#"{"hosts":{"github.com":[{"state":"error","active":false,"host":"github.com","login":"work"}]}}"#,
        )]);
        assert_eq!(
            GithubCliService::new(&unhealthy_runner).switch_account("github.com", "work"),
            Err(GithubCliError::AccountUnavailable)
        );
        assert_eq!(unhealthy_runner.calls().len(), 1);
    }

    #[test]
    fn reports_switch_process_failure_without_attempting_verification() {
        let runner = FakeProcessRunner::new(vec![
            successful(&status_json("personal")),
            failed("sensitive switch diagnostic"),
        ]);

        assert_eq!(
            GithubCliService::new(&runner).switch_account("github.com", "work"),
            Err(GithubCliError::SwitchFailed)
        );
        assert_eq!(runner.calls().len(), 2);
    }

    #[test]
    fn fails_when_switch_does_not_become_active() {
        let runner = FakeProcessRunner::new(vec![
            successful(&status_json("personal")),
            successful("switched"),
            successful(&status_json("personal")),
        ]);

        assert_eq!(
            GithubCliService::new(&runner).switch_account("github.com", "work"),
            Err(GithubCliError::SwitchNotVerified)
        );
    }

    #[test]
    fn does_not_launch_login_when_structured_status_is_unavailable() {
        let runner = FakeProcessRunner::new(vec![failed("unknown flag: --json")]);

        assert_eq!(
            GithubCliService::new(&runner).launch_login("github.com"),
            Err(GithubCliError::StatusFailed)
        );
        assert_eq!(
            runner.calls(),
            vec![(
                "gh".to_owned(),
                vec!["auth", "status", "--json", "hosts"]
                    .into_iter()
                    .map(str::to_owned)
                    .collect(),
            )]
        );
        assert!(runner.started_calls().is_empty());
    }

    #[test]
    fn starts_browser_login_without_blocking_or_token_flags() {
        let runner = FakeProcessRunner::new(vec![successful(&status_json("personal"))]);

        let status = GithubCliService::new(&runner)
            .launch_login("github.com")
            .unwrap();

        assert_eq!(status.accounts.len(), 2);
        assert_eq!(
            runner.calls(),
            vec![(
                "gh".to_owned(),
                vec!["auth", "status", "--json", "hosts"]
                    .into_iter()
                    .map(str::to_owned)
                    .collect(),
            )]
        );
        assert_eq!(
            runner.started_calls(),
            vec![(
                "gh".to_owned(),
                vec![
                    "auth",
                    "login",
                    "--hostname",
                    "github.com",
                    "--web",
                    "--clipboard",
                ]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            )]
        );
        assert!(runner.started_calls().iter().all(|(_, arguments)| {
            !arguments.iter().any(|argument| {
                argument == "token" || argument == "--show-token" || argument == "--with-token"
            })
        }));
    }

    #[test]
    fn reports_login_start_failure_without_waiting() {
        let runner = FakeProcessRunner::new(vec![successful(&status_json("personal"))])
            .with_start_error(ProcessError {
                kind: ProcessErrorKind::Other,
            });

        assert_eq!(
            GithubCliService::new(&runner).launch_login("github.com"),
            Err(GithubCliError::LoginFailed)
        );
        assert_eq!(runner.calls().len(), 1);
        assert_eq!(runner.started_calls().len(), 1);
    }

    #[test]
    fn validates_login_hosts_and_switch_usernames_before_running_commands() {
        let runner = FakeProcessRunner::default();

        assert_eq!(
            GithubCliService::new(&runner).launch_login("github.com --with-token"),
            Err(GithubCliError::InvalidHostname)
        );
        assert_eq!(
            GithubCliService::new(&runner).switch_account("github.com", "-invalid"),
            Err(GithubCliError::InvalidUsername)
        );
        assert!(runner.calls().is_empty());
    }
}
