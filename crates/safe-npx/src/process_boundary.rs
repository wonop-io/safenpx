//! Reviewed process execution boundary.
//!
//! Production modules should not use `std::process::Command` directly. Route
//! process launch through this file so policy checks have one auditable edge.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

/// Direct process invocation with no shell interpolation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessInvocation {
    /// Executable path passed directly to the operating system.
    pub executable: PathBuf,
    /// Arguments passed as discrete argv values.
    pub args: Vec<String>,
    /// Working directory for the child process.
    pub cwd: PathBuf,
    /// Explicit environment variables for the child process.
    pub environment: BTreeMap<String, String>,
}

/// Captured process output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessBoundaryOutput {
    /// Process exit code when the platform reports one.
    pub exit_code: Option<i32>,
    /// Captured standard output bytes.
    pub stdout: Vec<u8>,
    /// Captured standard error bytes.
    pub stderr: Vec<u8>,
}

/// Execute one direct process invocation without shell fallback.
pub fn run_direct_process(
    invocation: &ProcessInvocation,
) -> Result<ProcessBoundaryOutput, std::io::Error> {
    let output = Command::new(&invocation.executable)
        .args(&invocation.args)
        .current_dir(&invocation.cwd)
        .env_clear()
        .envs(invocation.environment.iter())
        .output()?;

    Ok(ProcessBoundaryOutput {
        exit_code: output.status.code(),
        stdout: output.stdout,
        stderr: output.stderr,
    })
}
