use std::{collections::HashSet, path::Path};

use serde::Serialize;
use url::Url;

use crate::process::{ProcessErrorKind, ProcessOutput, ProcessRunner};

const MAX_PATH_CHARS: usize = 32_768;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RepositoryInspection {
    pub(crate) path: String,
    pub(crate) remotes: Vec<GitRemote>,
    pub(crate) identity: GitIdentity,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GitRemote {
    pub(crate) name: String,
    pub(crate) url: String,
    pub(crate) direction: RemoteDirection,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum RemoteDirection {
    Fetch,
    Push,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GitIdentity {
    pub(crate) name: GitConfigValue,
    pub(crate) email: GitConfigValue,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GitConfigValue {
    pub(crate) value: Option<String>,
    pub(crate) source: Option<GitConfigSource>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GitConfigSource {
    pub(crate) scope: GitConfigScope,
    pub(crate) origin: String,
    pub(crate) conditional_include: bool,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum GitConfigScope {
    System,
    Global,
    Local,
    Worktree,
    Command,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GitInspectionError {
    InvalidPath,
    PathNotFound,
    NotDirectory,
    GitMissing,
    NotRepository,
    CommandFailed,
    MalformedOutput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GitIdentityApplyError {
    Inspection(GitInspectionError),
    InvalidIdentity,
    WriteFailed,
    VerificationFailed,
    RollbackFailed,
}

pub(crate) struct GitService<R> {
    runner: R,
}

impl<R: ProcessRunner> GitService<R> {
    pub(crate) fn new(runner: R) -> Self {
        Self { runner }
    }

    pub(crate) fn inspect(
        &self,
        selected_path: &str,
    ) -> Result<RepositoryInspection, GitInspectionError> {
        validate_path_input(selected_path)?;
        let selected = dunce::canonicalize(selected_path).map_err(|error| match error.kind() {
            std::io::ErrorKind::NotFound => GitInspectionError::PathNotFound,
            _ => GitInspectionError::InvalidPath,
        })?;
        if !selected.is_dir() {
            return Err(GitInspectionError::NotDirectory);
        }
        let selected_text = path_to_string(&selected)?;

        let top_level_output =
            self.run_git(&["-C", selected_text.as_str(), "rev-parse", "--show-toplevel"])?;
        if !top_level_output.success {
            return Err(GitInspectionError::NotRepository);
        }

        let reported_root = parse_single_line(&top_level_output.stdout)?;
        let root =
            dunce::canonicalize(reported_root).map_err(|_| GitInspectionError::MalformedOutput)?;
        if !root.is_dir() || !selected.starts_with(&root) {
            return Err(GitInspectionError::MalformedOutput);
        }
        let root_text = path_to_string(&root)?;

        let remotes_output = self.run_git(&["-C", root_text.as_str(), "remote", "-v"])?;
        if !remotes_output.success {
            return Err(GitInspectionError::CommandFailed);
        }
        let remotes = parse_remotes(&remotes_output.stdout)?;

        let conditional_origins = self.conditional_include_origins(&root_text)?;
        let name = self.config_value(&root_text, "user.name", &conditional_origins)?;
        let email = self.config_value(&root_text, "user.email", &conditional_origins)?;

        Ok(RepositoryInspection {
            path: root_text,
            remotes,
            identity: GitIdentity { name, email },
        })
    }

    pub(crate) fn apply_local_identity(
        &self,
        repository_path: &str,
        name: &str,
        email: &str,
    ) -> Result<RepositoryInspection, GitIdentityApplyError> {
        validate_identity_value(name)?;
        validate_identity_value(email)?;

        let current = self
            .inspect(repository_path)
            .map_err(GitIdentityApplyError::Inspection)?;
        let root = current.path.as_str();
        let previous_name = self
            .local_values(root, "user.name")
            .map_err(GitIdentityApplyError::Inspection)?;
        let previous_email = self
            .local_values(root, "user.email")
            .map_err(GitIdentityApplyError::Inspection)?;

        if self.write_local_value(root, "user.name", name).is_err()
            || self.write_local_value(root, "user.email", email).is_err()
        {
            return Err(self.rollback_error(root, &previous_name, &previous_email, false));
        }

        let verified = self.inspect(root).ok().filter(|inspection| {
            config_matches_local(&inspection.identity.name, name)
                && config_matches_local(&inspection.identity.email, email)
                && self.local_values(root, "user.name").ok() == Some(vec![name.to_owned()])
                && self.local_values(root, "user.email").ok() == Some(vec![email.to_owned()])
        });

        match verified {
            Some(inspection) => Ok(inspection),
            None => Err(self.rollback_error(root, &previous_name, &previous_email, true)),
        }
    }

    fn rollback_error(
        &self,
        root: &str,
        previous_name: &[String],
        previous_email: &[String],
        verification_failed: bool,
    ) -> GitIdentityApplyError {
        let name_restored = self.restore_local_values(root, "user.name", previous_name);
        let email_restored = self.restore_local_values(root, "user.email", previous_email);
        if name_restored && email_restored {
            if verification_failed {
                GitIdentityApplyError::VerificationFailed
            } else {
                GitIdentityApplyError::WriteFailed
            }
        } else {
            GitIdentityApplyError::RollbackFailed
        }
    }

    fn local_values(&self, root: &str, key: &str) -> Result<Vec<String>, GitInspectionError> {
        let output = self.run_git(&[
            "-C",
            root,
            "config",
            "--local",
            "--no-includes",
            "--null",
            "--get-all",
            key,
        ])?;
        if !output.success {
            if matches!(output.exit_code, Some(1 | 5)) && output.stdout.is_empty() {
                return Ok(Vec::new());
            }
            return Err(GitInspectionError::CommandFailed);
        }
        Ok(null_fields(&output.stdout)
            .into_iter()
            .map(str::to_owned)
            .collect())
    }

    fn write_local_value(
        &self,
        root: &str,
        key: &str,
        value: &str,
    ) -> Result<(), GitInspectionError> {
        let output = self.run_git(&[
            "-C",
            root,
            "config",
            "--local",
            "--no-includes",
            "--replace-all",
            key,
            value,
        ])?;
        if output.success {
            Ok(())
        } else {
            Err(GitInspectionError::CommandFailed)
        }
    }

    fn restore_local_values(&self, root: &str, key: &str, values: &[String]) -> bool {
        let unset = self.run_git(&[
            "-C",
            root,
            "config",
            "--local",
            "--no-includes",
            "--unset-all",
            key,
        ]);
        if !matches!(
            unset,
            Ok(ProcessOutput { success: true, .. })
                | Ok(ProcessOutput {
                    success: false,
                    exit_code: Some(1 | 5),
                    ..
                })
        ) {
            return false;
        }

        values.iter().all(|value| {
            self.run_git(&[
                "-C",
                root,
                "config",
                "--local",
                "--no-includes",
                "--add",
                key,
                value,
            ])
            .is_ok_and(|output| output.success)
        })
    }

    fn config_value(
        &self,
        root: &str,
        key: &str,
        conditional_origins: &HashSet<String>,
    ) -> Result<GitConfigValue, GitInspectionError> {
        let output = self.run_git(&[
            "-C",
            root,
            "config",
            "--null",
            "--show-origin",
            "--show-scope",
            "--get",
            key,
        ])?;

        if !output.success {
            if output.exit_code == Some(1) && output.stdout.is_empty() {
                return Ok(GitConfigValue {
                    value: None,
                    source: None,
                });
            }
            return Err(GitInspectionError::CommandFailed);
        }

        let (scope, origin, value) = parse_config_value(&output.stdout)?;
        Ok(GitConfigValue {
            value: Some(value),
            source: Some(GitConfigSource {
                scope,
                conditional_include: conditional_origins.contains(&normalize_origin(&origin)),
                origin,
            }),
        })
    }

    fn conditional_include_origins(
        &self,
        root: &str,
    ) -> Result<HashSet<String>, GitInspectionError> {
        let output = self.run_git(&[
            "-C",
            root,
            "config",
            "--path",
            "--null",
            "--show-origin",
            "--show-scope",
            "--get-regexp",
            "^includeIf\\..*\\.path$",
        ])?;

        if !output.success {
            if output.exit_code == Some(1) && output.stdout.is_empty() {
                return Ok(HashSet::new());
            }
            return Err(GitInspectionError::CommandFailed);
        }

        parse_conditional_include_origins(&output.stdout)
    }

    fn run_git(&self, args: &[&str]) -> Result<ProcessOutput, GitInspectionError> {
        self.runner
            .run("git", args)
            .map_err(|error| match error.kind {
                ProcessErrorKind::NotFound => GitInspectionError::GitMissing,
                ProcessErrorKind::PermissionDenied | ProcessErrorKind::Other => {
                    GitInspectionError::CommandFailed
                }
            })
    }
}

fn validate_identity_value(value: &str) -> Result<(), GitIdentityApplyError> {
    if value.is_empty() || value.chars().count() > 254 || value.chars().any(char::is_control) {
        Err(GitIdentityApplyError::InvalidIdentity)
    } else {
        Ok(())
    }
}

fn config_matches_local(value: &GitConfigValue, expected: &str) -> bool {
    value.value.as_deref() == Some(expected)
        && value
            .source
            .as_ref()
            .is_some_and(|source| source.scope == GitConfigScope::Local)
}

fn validate_path_input(path: &str) -> Result<(), GitInspectionError> {
    if path.is_empty()
        || path.chars().count() > MAX_PATH_CHARS
        || path.chars().any(|character| character == '\0')
    {
        Err(GitInspectionError::InvalidPath)
    } else {
        Ok(())
    }
}

fn path_to_string(path: &Path) -> Result<String, GitInspectionError> {
    path.to_str()
        .map(str::to_owned)
        .ok_or(GitInspectionError::InvalidPath)
}

fn parse_single_line(output: &str) -> Result<&str, GitInspectionError> {
    let mut lines = output.lines().filter(|line| !line.is_empty());
    let value = lines.next().ok_or(GitInspectionError::MalformedOutput)?;
    if lines.next().is_some() || value.contains('\0') {
        return Err(GitInspectionError::MalformedOutput);
    }
    Ok(value)
}

fn parse_remotes(output: &str) -> Result<Vec<GitRemote>, GitInspectionError> {
    let mut remotes = Vec::new();
    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let (body, direction) = if let Some(body) = line.strip_suffix(" (fetch)") {
            (body, RemoteDirection::Fetch)
        } else if let Some(body) = line.strip_suffix(" (push)") {
            (body, RemoteDirection::Push)
        } else {
            return Err(GitInspectionError::MalformedOutput);
        };
        let split_at = body
            .find(char::is_whitespace)
            .ok_or(GitInspectionError::MalformedOutput)?;
        let name = &body[..split_at];
        let url = body[split_at..].trim();
        if name.is_empty() || url.is_empty() || name.chars().any(char::is_control) {
            return Err(GitInspectionError::MalformedOutput);
        }
        remotes.push(GitRemote {
            name: name.to_owned(),
            url: redact_remote_url(url),
            direction,
        });
    }
    Ok(remotes)
}

fn redact_remote_url(value: &str) -> String {
    if let Ok(mut url) = Url::parse(value) {
        if (!url.username().is_empty() || url.password().is_some())
            && (url.set_username("").is_err() || url.set_password(None).is_err())
        {
            return "<redacted remote>".to_owned();
        }
        return url.to_string();
    }

    if let Some(at) = value.rfind('@') {
        let suffix = &value[at + 1..];
        if suffix.contains(':') {
            return suffix.to_owned();
        }
    }

    value.to_owned()
}

fn parse_config_value(
    output: &str,
) -> Result<(GitConfigScope, String, String), GitInspectionError> {
    let fields = null_fields(output);
    if fields.len() != 3 {
        return Err(GitInspectionError::MalformedOutput);
    }
    let scope = parse_scope(fields[0]);
    let origin = fields[1].to_owned();
    let value = fields[2].to_owned();
    if origin.is_empty() {
        return Err(GitInspectionError::MalformedOutput);
    }
    Ok((scope, origin, value))
}

fn parse_scope(value: &str) -> GitConfigScope {
    match value {
        "system" => GitConfigScope::System,
        "global" => GitConfigScope::Global,
        "local" => GitConfigScope::Local,
        "worktree" => GitConfigScope::Worktree,
        "command" => GitConfigScope::Command,
        _ => GitConfigScope::Unknown,
    }
}

fn parse_conditional_include_origins(output: &str) -> Result<HashSet<String>, GitInspectionError> {
    let fields = null_fields(output);
    let (entries, remainder) = fields.as_chunks::<3>();
    if !remainder.is_empty() {
        return Err(GitInspectionError::MalformedOutput);
    }

    let mut origins = HashSet::new();
    for entry in entries {
        let including_origin = entry[1];
        let (name, target) = entry[2]
            .split_once('\n')
            .ok_or(GitInspectionError::MalformedOutput)?;
        if !name.to_ascii_lowercase().starts_with("includeif.") || target.is_empty() {
            return Err(GitInspectionError::MalformedOutput);
        }
        origins.insert(resolve_include_origin(including_origin, target));
    }
    Ok(origins)
}

fn resolve_include_origin(including_origin: &str, target: &str) -> String {
    if target.starts_with("~/") {
        return target.to_owned();
    }

    let target_path = Path::new(target);
    if target_path.is_absolute() {
        return target_path.to_string_lossy().into_owned();
    }

    let including_path = normalize_origin(including_origin);
    Path::new(&including_path)
        .parent()
        .map(|parent| parent.join(target_path).to_string_lossy().into_owned())
        .unwrap_or_else(|| target.to_owned())
}

fn normalize_origin(origin: &str) -> String {
    origin.strip_prefix("file:").unwrap_or(origin).to_owned()
}

fn null_fields(output: &str) -> Vec<&str> {
    let mut fields: Vec<_> = output.split('\0').collect();
    if fields.last() == Some(&"") {
        fields.pop();
    }
    fields
}

#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, fs, path::PathBuf, sync::Mutex};

    use super::*;
    use crate::process::{ProcessError, ProcessOutput};

    struct FakeRunner {
        responses: Mutex<VecDeque<Result<ProcessOutput, ProcessError>>>,
        calls: Mutex<Vec<(String, Vec<String>)>>,
    }

    impl FakeRunner {
        fn new(responses: Vec<Result<ProcessOutput, ProcessError>>) -> Self {
            Self {
                responses: Mutex::new(responses.into()),
                calls: Mutex::new(Vec::new()),
            }
        }
    }

    impl ProcessRunner for &FakeRunner {
        fn run(&self, program: &str, args: &[&str]) -> Result<ProcessOutput, ProcessError> {
            self.calls.lock().unwrap().push((
                program.to_owned(),
                args.iter().map(|argument| (*argument).to_owned()).collect(),
            ));
            self.responses.lock().unwrap().pop_front().unwrap()
        }

        fn start(&self, _program: &str, _args: &[&str]) -> Result<(), ProcessError> {
            panic!("repository inspection must not start a process")
        }
    }

    fn success(stdout: &str) -> Result<ProcessOutput, ProcessError> {
        Ok(ProcessOutput {
            success: true,
            exit_code: Some(0),
            stdout: stdout.to_owned(),
        })
    }

    #[test]
    fn parses_and_redacts_remote_lines() {
        let remotes = parse_remotes(
            "origin\thttps://user:token@github.com/org/repo.git (fetch)\norigin\tgit@github.com:org/repo.git (push)\n",
        )
        .unwrap();

        assert_eq!(remotes.len(), 2);
        assert_eq!(remotes[0].url, "https://github.com/org/repo.git");
        assert_eq!(remotes[1].url, "github.com:org/repo.git");
        assert!(!format!("{remotes:?}").contains("token"));
    }

    #[test]
    fn parses_null_delimited_config_values() {
        let parsed = parse_config_value("global\0file:/tmp/config\0Octo Cat\0").unwrap();
        assert_eq!(parsed.0, GitConfigScope::Global);
        assert_eq!(parsed.1, "file:/tmp/config");
        assert_eq!(parsed.2, "Octo Cat");
        assert_eq!(
            parse_config_value("global\0file:/tmp/config\0\0"),
            Ok((
                GitConfigScope::Global,
                "file:/tmp/config".to_owned(),
                String::new()
            ))
        );
    }

    #[test]
    fn identifies_conditional_include_targets() {
        let origins = parse_conditional_include_origins(
            "global\0file:/tmp/global\0includeif.gitdir:/tmp/work/.path\nidentity.inc\0",
        )
        .unwrap();
        assert!(origins.contains("/tmp/identity.inc"));
    }

    #[test]
    fn rejects_malformed_remote_and_config_output() {
        assert_eq!(
            parse_remotes("origin missing-direction"),
            Err(GitInspectionError::MalformedOutput)
        );
        assert_eq!(
            parse_config_value("global\0file:/tmp/config\0name\0extra\0"),
            Err(GitInspectionError::MalformedOutput)
        );
    }

    #[test]
    fn validates_paths_before_running_git() {
        let runner = FakeRunner::new(Vec::new());
        assert_eq!(
            GitService::new(&runner).inspect(""),
            Err(GitInspectionError::InvalidPath)
        );
        assert!(runner.calls.lock().unwrap().is_empty());
    }

    #[test]
    fn uses_only_fixed_git_argument_arrays() {
        let directory = TestDirectory::new();
        let root = directory.path().to_string_lossy().into_owned();
        let canonical_root = fs::canonicalize(directory.path())
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let runner = FakeRunner::new(vec![
            success(&format!("{root}\n")),
            success("origin\thttps://github.com/octo/project.git (fetch)\n"),
            failed_empty(),
            success("global\0file:/tmp/global\0Octo Cat\0"),
            failed_empty(),
        ]);

        let inspection = GitService::new(&runner).inspect(&root).unwrap();

        assert_eq!(inspection.identity.name.value.as_deref(), Some("Octo Cat"));
        assert_eq!(inspection.identity.email.value, None);
        let calls = runner.calls.lock().unwrap();
        assert!(calls.iter().all(|(program, _)| program == "git"));
        assert_eq!(
            calls[0].1,
            vec![
                "-C",
                canonical_root.as_str(),
                "rev-parse",
                "--show-toplevel"
            ]
        );
        assert!(calls.iter().all(|(_, arguments)| {
            !arguments
                .iter()
                .any(|argument| argument == "-c" || argument == "sh")
        }));
    }

    #[test]
    fn reports_process_failures_without_raw_diagnostics() {
        let directory = TestDirectory::new();
        let root = directory.path().to_string_lossy().into_owned();
        let runner = FakeRunner::new(vec![
            success(&format!("{root}\n")),
            Ok(ProcessOutput {
                success: false,
                exit_code: Some(2),
                stdout: "remote URL containing a secret".to_owned(),
            }),
        ]);

        assert_eq!(
            GitService::new(&runner).inspect(&root),
            Err(GitInspectionError::CommandFailed)
        );
    }

    #[test]
    fn reports_missing_git_and_malformed_output_without_raw_diagnostics() {
        let directory = TestDirectory::new();
        let missing = FakeRunner::new(vec![Err(ProcessError {
            kind: ProcessErrorKind::NotFound,
        })]);
        assert_eq!(
            GitService::new(&missing).inspect(directory.path().to_str().unwrap()),
            Err(GitInspectionError::GitMissing)
        );

        let root = directory.path().to_string_lossy().into_owned();
        let malformed = FakeRunner::new(vec![success(&format!("{root}\n")), success("secret")]);
        assert_eq!(
            GitService::new(&malformed).inspect(&root),
            Err(GitInspectionError::MalformedOutput)
        );
    }

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "git-identity-manager-git-test-{}",
                uuid::Uuid::new_v4()
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

    struct IsolatedGitRunner {
        home: PathBuf,
        global_config: PathBuf,
    }

    impl ProcessRunner for &IsolatedGitRunner {
        fn run(&self, program: &str, args: &[&str]) -> Result<ProcessOutput, ProcessError> {
            let output = std::process::Command::new(program)
                .args(args)
                .env("HOME", &self.home)
                .env("GIT_CONFIG_GLOBAL", &self.global_config)
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .output()
                .map_err(ProcessError::from)?;
            Ok(ProcessOutput {
                success: output.status.success(),
                exit_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            })
        }

        fn start(&self, _program: &str, _args: &[&str]) -> Result<(), ProcessError> {
            panic!("repository inspection must not start a process")
        }
    }

    fn run_git(directory: &Path, arguments: &[&str]) {
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(directory)
            .args(arguments)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .output()
            .unwrap();
        assert!(output.status.success());
    }

    #[test]
    fn inspects_disposable_repository_with_inherited_and_local_identity() {
        let directory = TestDirectory::new();
        let repository = directory.path().join("repository");
        fs::create_dir(&repository).unwrap();
        run_git(&repository, &["init", "--quiet"]);
        let nested = repository.join("nested");
        fs::create_dir(&nested).unwrap();
        let global = directory.path().join("global.gitconfig");
        fs::write(
            &global,
            "[user]\n\tname = Global Octo\n\temail = global@example.com\n",
        )
        .unwrap();
        let runner = IsolatedGitRunner {
            home: directory.path().to_owned(),
            global_config: global,
        };

        let inherited = GitService::new(&runner)
            .inspect(nested.to_str().unwrap())
            .unwrap();
        assert_eq!(
            inherited.path,
            dunce::canonicalize(&repository).unwrap().to_string_lossy()
        );
        assert_eq!(
            inherited.identity.name.value.as_deref(),
            Some("Global Octo")
        );
        assert_eq!(
            inherited.identity.name.source.unwrap().scope,
            GitConfigScope::Global
        );
        assert!(inherited.remotes.is_empty());

        run_git(
            &repository,
            &["config", "--local", "user.email", "local@example.com"],
        );
        run_git(
            &repository,
            &[
                "remote",
                "add",
                "origin",
                "https://github.com/octo/repository.git",
            ],
        );
        let local = GitService::new(&runner)
            .inspect(repository.to_str().unwrap())
            .unwrap();
        assert_eq!(
            local.identity.email.value.as_deref(),
            Some("local@example.com")
        );
        assert_eq!(
            local.identity.email.source.unwrap().scope,
            GitConfigScope::Local
        );
        assert_eq!(local.remotes.len(), 2);
    }

    #[test]
    fn reports_unset_identity_without_an_origin() {
        let directory = TestDirectory::new();
        let repository = directory.path().join("empty-repository");
        fs::create_dir(&repository).unwrap();
        run_git(&repository, &["init", "--quiet"]);
        let global = directory.path().join("empty-global.gitconfig");
        fs::write(&global, "").unwrap();
        let runner = IsolatedGitRunner {
            home: directory.path().to_owned(),
            global_config: global,
        };

        let inspection = GitService::new(&runner)
            .inspect(repository.to_str().unwrap())
            .unwrap();
        assert_eq!(inspection.identity.name.value, None);
        assert_eq!(inspection.identity.name.source, None);
        assert_eq!(inspection.identity.email.value, None);
        assert_eq!(inspection.identity.email.source, None);
    }

    #[test]
    fn identifies_an_active_conditional_include_origin() {
        let directory = TestDirectory::new();
        let repository = directory.path().join("conditional-repository");
        fs::create_dir(&repository).unwrap();
        run_git(&repository, &["init", "--quiet"]);
        let canonical_git_directory = fs::canonicalize(repository.join(".git")).unwrap();
        let included = directory.path().join("identity.inc");
        fs::write(&included, "[user]\n\tname = Conditional Octo\n").unwrap();
        let global = directory.path().join("conditional-global.gitconfig");
        fs::write(
            &global,
            format!(
                "[includeIf \"gitdir:{}\"]\n\tpath = {}\n",
                canonical_git_directory.to_string_lossy(),
                included.to_string_lossy()
            ),
        )
        .unwrap();
        let runner = IsolatedGitRunner {
            home: directory.path().to_owned(),
            global_config: global,
        };

        let inspection = GitService::new(&runner)
            .inspect(repository.to_str().unwrap())
            .unwrap();
        let source = inspection.identity.name.source.unwrap();
        assert_eq!(
            inspection.identity.name.value.as_deref(),
            Some("Conditional Octo")
        );
        assert!(source.conditional_include);
        assert_eq!(normalize_origin(&source.origin), included.to_string_lossy());
    }

    #[test]
    fn resolves_a_linked_worktree_top_level() {
        let directory = TestDirectory::new();
        let repository = directory.path().join("main");
        let worktree = directory.path().join("linked");
        fs::create_dir(&repository).unwrap();
        run_git(&repository, &["init", "--quiet"]);
        run_git(
            &repository,
            &[
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.com",
                "commit",
                "--allow-empty",
                "--quiet",
                "-m",
                "initial",
            ],
        );
        run_git(
            &repository,
            &["worktree", "add", "--quiet", worktree.to_str().unwrap()],
        );
        let global = directory.path().join("worktree-global.gitconfig");
        fs::write(&global, "").unwrap();
        let runner = IsolatedGitRunner {
            home: directory.path().to_owned(),
            global_config: global,
        };

        let inspection = GitService::new(&runner)
            .inspect(worktree.to_str().unwrap())
            .unwrap();
        assert_eq!(
            inspection.path,
            fs::canonicalize(&worktree).unwrap().to_string_lossy()
        );
    }

    #[test]
    fn applies_and_verifies_only_repository_local_identity() {
        let directory = TestDirectory::new();
        let repository = directory.path().join("apply-repository");
        fs::create_dir(&repository).unwrap();
        run_git(&repository, &["init", "--quiet"]);
        run_git(
            &repository,
            &["config", "--local", "core.testSetting", "preserved"],
        );
        let global = directory.path().join("apply-global.gitconfig");
        fs::write(
            &global,
            "[user]\n\tname = Global Octo\n\temail = global@example.com\n",
        )
        .unwrap();
        let global_before = fs::read(&global).unwrap();
        let runner = IsolatedGitRunner {
            home: directory.path().to_owned(),
            global_config: global.clone(),
        };

        let inspection = GitService::new(&runner)
            .apply_local_identity(
                &repository.to_string_lossy(),
                "Work Octo",
                "work@example.com",
            )
            .unwrap();

        assert!(config_matches_local(&inspection.identity.name, "Work Octo"));
        assert!(config_matches_local(
            &inspection.identity.email,
            "work@example.com"
        ));
        assert_eq!(fs::read(global).unwrap(), global_before);
        let preserved = std::process::Command::new("git")
            .args([
                "-C",
                repository.to_str().unwrap(),
                "config",
                "--local",
                "--get",
                "core.testSetting",
            ])
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .output()
            .unwrap();
        assert!(preserved.status.success());
        assert_eq!(
            String::from_utf8_lossy(&preserved.stdout).trim(),
            "preserved"
        );
    }

    struct CommandOverrideGitRunner {
        inner: IsolatedGitRunner,
    }

    impl ProcessRunner for &CommandOverrideGitRunner {
        fn run(&self, program: &str, args: &[&str]) -> Result<ProcessOutput, ProcessError> {
            let output = std::process::Command::new(program)
                .args(args)
                .env("HOME", &self.inner.home)
                .env("GIT_CONFIG_GLOBAL", &self.inner.global_config)
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_COUNT", "2")
                .env("GIT_CONFIG_KEY_0", "user.name")
                .env("GIT_CONFIG_VALUE_0", "Command Name")
                .env("GIT_CONFIG_KEY_1", "user.email")
                .env("GIT_CONFIG_VALUE_1", "command@example.com")
                .output()
                .map_err(ProcessError::from)?;
            Ok(ProcessOutput {
                success: output.status.success(),
                exit_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            })
        }

        fn start(&self, _program: &str, _args: &[&str]) -> Result<(), ProcessError> {
            panic!("repository identity application must not start a process")
        }
    }

    #[test]
    fn restores_previous_local_values_when_verification_fails() {
        let directory = TestDirectory::new();
        let repository = directory.path().join("rollback-repository");
        fs::create_dir(&repository).unwrap();
        run_git(&repository, &["init", "--quiet"]);
        run_git(
            &repository,
            &["config", "--local", "user.name", "Previous Name"],
        );
        run_git(
            &repository,
            &["config", "--local", "user.email", "previous@example.com"],
        );
        let global = directory.path().join("rollback-global.gitconfig");
        fs::write(&global, "").unwrap();
        let runner = CommandOverrideGitRunner {
            inner: IsolatedGitRunner {
                home: directory.path().to_owned(),
                global_config: global,
            },
        };

        assert_eq!(
            GitService::new(&runner).apply_local_identity(
                &repository.to_string_lossy(),
                "Desired Name",
                "desired@example.com"
            ),
            Err(GitIdentityApplyError::VerificationFailed)
        );

        let local_name = std::process::Command::new("git")
            .args([
                "-C",
                repository.to_str().unwrap(),
                "config",
                "--local",
                "--get",
                "user.name",
            ])
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .output()
            .unwrap();
        let local_email = std::process::Command::new("git")
            .args([
                "-C",
                repository.to_str().unwrap(),
                "config",
                "--local",
                "--get",
                "user.email",
            ])
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .output()
            .unwrap();
        assert_eq!(
            String::from_utf8_lossy(&local_name.stdout).trim(),
            "Previous Name"
        );
        assert_eq!(
            String::from_utf8_lossy(&local_email.stdout).trim(),
            "previous@example.com"
        );
    }

    #[test]
    fn rejects_identity_values_before_running_git() {
        let runner = FakeRunner::new(Vec::new());
        assert_eq!(
            GitService::new(&runner).apply_local_identity(
                "/work/project",
                "bad\nname",
                "ok@example.com"
            ),
            Err(GitIdentityApplyError::InvalidIdentity)
        );
        assert!(runner.calls.lock().unwrap().is_empty());
    }

    #[test]
    fn rejects_a_disposable_non_git_directory() {
        let directory = TestDirectory::new();
        let global = directory.path().join("global.gitconfig");
        fs::write(&global, "").unwrap();
        let runner = IsolatedGitRunner {
            home: directory.path().to_owned(),
            global_config: global,
        };

        assert_eq!(
            GitService::new(&runner).inspect(directory.path().to_str().unwrap()),
            Err(GitInspectionError::NotRepository)
        );
    }

    fn failed_empty() -> Result<ProcessOutput, ProcessError> {
        Ok(ProcessOutput {
            success: false,
            exit_code: Some(1),
            stdout: String::new(),
        })
    }
}
