use std::{
    env,
    ffi::OsString,
    io,
    path::PathBuf,
    process::{Command, Stdio},
    sync::OnceLock,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProcessOutput {
    pub(crate) success: bool,
    pub(crate) exit_code: Option<i32>,
    pub(crate) stdout: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProcessErrorKind {
    NotFound,
    PermissionDenied,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProcessError {
    pub(crate) kind: ProcessErrorKind,
}

pub(crate) trait ProcessRunner: Send + Sync {
    fn run(&self, program: &str, args: &[&str]) -> Result<ProcessOutput, ProcessError>;
    fn start(&self, program: &str, args: &[&str]) -> Result<(), ProcessError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct SystemProcessRunner;

impl ProcessRunner for SystemProcessRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<ProcessOutput, ProcessError> {
        let mut command = Command::new(program);
        command.args(args);
        apply_resolved_path(&mut command);
        command
            .output()
            .map(|output| ProcessOutput {
                success: output.status.success(),
                exit_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            })
            .map_err(ProcessError::from)
    }

    fn start(&self, program: &str, args: &[&str]) -> Result<(), ProcessError> {
        let mut command = Command::new(program);
        command
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        apply_resolved_path(&mut command);
        let mut child = command.spawn().map_err(ProcessError::from)?;

        std::thread::spawn(move || {
            let _ = child.wait();
        });

        Ok(())
    }
}

/// Directories that commonly hold `git`/`gh` but are missing from the minimal `PATH`
/// a GUI application inherits when launched from Finder/Dock rather than a shell.
#[cfg(not(target_os = "windows"))]
const EXTRA_PATH_DIRS: &[&str] = &[
    "/opt/homebrew/bin",
    "/opt/homebrew/sbin",
    "/usr/local/bin",
    "/usr/local/sbin",
    "/usr/bin",
    "/bin",
    "/usr/sbin",
    "/sbin",
    "/opt/local/bin",
    "/snap/bin",
];

#[cfg(target_os = "windows")]
const EXTRA_PATH_DIRS: &[&str] = &[];

fn apply_resolved_path(command: &mut Command) {
    static RESOLVED_PATH: OnceLock<Option<OsString>> = OnceLock::new();
    if let Some(path) = RESOLVED_PATH.get_or_init(|| resolved_path(env::var_os("PATH"))) {
        command.env("PATH", path);
    }
}

/// Builds a `PATH` value that keeps every inherited entry and appends the well-known
/// install locations that are not already present and actually exist. Returns `None`
/// when nothing needs to change so the process inherits `PATH` untouched.
fn resolved_path(current: Option<OsString>) -> Option<OsString> {
    let mut dirs: Vec<PathBuf> = current
        .as_ref()
        .map(|value| env::split_paths(value).collect())
        .unwrap_or_default();

    let mut candidates: Vec<PathBuf> = EXTRA_PATH_DIRS.iter().map(PathBuf::from).collect();
    if let Some(home) = env::var_os("HOME").filter(|home| !home.is_empty()) {
        candidates.push(PathBuf::from(&home).join(".local").join("bin"));
        candidates.push(PathBuf::from(&home).join("bin"));
    }

    let mut changed = false;
    for candidate in candidates {
        if candidate.is_dir() && !dirs.contains(&candidate) {
            dirs.push(candidate);
            changed = true;
        }
    }

    if !changed {
        return None;
    }
    env::join_paths(dirs).ok()
}

impl From<io::Error> for ProcessError {
    fn from(error: io::Error) -> Self {
        let kind = match error.kind() {
            io::ErrorKind::NotFound => ProcessErrorKind::NotFound,
            io::ErrorKind::PermissionDenied => ProcessErrorKind::PermissionDenied,
            _ => ProcessErrorKind::Other,
        };

        Self { kind }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolved_path_keeps_inherited_entries_first_and_appends_known_dirs() {
        let resolved = resolved_path(Some(OsString::from("/definitely/not/real")))
            .expect("known bin dirs on this host should extend a minimal PATH");
        let entries: Vec<PathBuf> = env::split_paths(&resolved).collect();

        assert_eq!(
            entries.first(),
            Some(&PathBuf::from("/definitely/not/real"))
        );
        // Every appended dir actually exists, and at least one was appended.
        assert!(entries.len() > 1);
        assert!(entries.iter().skip(1).all(|dir| dir.is_dir()));
    }

    #[test]
    fn resolved_path_does_not_duplicate_dirs_already_present() {
        let bin = if PathBuf::from("/usr/bin").is_dir() {
            "/usr/bin"
        } else {
            return;
        };
        let resolved = resolved_path(Some(OsString::from(bin))).unwrap_or_default();
        let count = env::split_paths(&resolved)
            .filter(|dir| dir == &PathBuf::from(bin))
            .count();
        assert_eq!(count, 1);
    }
}
