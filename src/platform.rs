//! Platform-specific process primitives.

use std::io;
use std::process::Command;

/// Replace the current process with `cmd`, or on Windows exit with the
/// child's code.
///
/// - Unix: calls `execvp` via `CommandExt::exec`. Does not return on success
///   (kernel replaces the image). Returns `io::Error` only on failure.
/// - Windows: no `execvp` equivalent. Spawns the child, waits, then calls
///   `std::process::exit` with the child's exit code. Does not return on
///   success (this function exits the process). Returns `io::Error` only on
///   spawn failure.
///
/// Callers should treat the return as a "didn't happen" error — the common
/// idiom is `Err(exec_replace(cmd))` and the `Err` is only ever constructed
/// in the failure path.
#[cfg(unix)]
pub fn exec_replace(mut cmd: Command) -> io::Error {
    use std::os::unix::process::CommandExt;
    cmd.exec()
}

#[cfg(windows)]
pub fn exec_replace(mut cmd: Command) -> io::Error {
    match cmd.status() {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(e) => e,
    }
}
