use std::{
    io,
    process::{Command, Stdio},
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
        Command::new(program)
            .args(args)
            .output()
            .map(|output| ProcessOutput {
                success: output.status.success(),
                exit_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            })
            .map_err(ProcessError::from)
    }

    fn start(&self, program: &str, args: &[&str]) -> Result<(), ProcessError> {
        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(ProcessError::from)?;

        std::thread::spawn(move || {
            let _ = child.wait();
        });

        Ok(())
    }
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
