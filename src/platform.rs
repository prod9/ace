//! Platform-specific process primitives.

use std::io;
use std::process::Command;

/// Replace the current process with `cmd`.
///
/// On Unix, uses `execvp` via `CommandExt::exec`; does not return on success.
/// On Windows, spawns the child, waits, and exits the parent with the child's
/// code — `execvp` has no native equivalent there.
///
/// Returns the `io::Error` from the underlying call on failure.
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
