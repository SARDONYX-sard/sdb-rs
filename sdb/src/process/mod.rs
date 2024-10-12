use crate::error::{
    CouldNotAttachSnafu, CouldNotCreatePipeSnafu, CouldNotResumeSnafu, NullSnafu, Result, SdbError,
    TracingFailedSnafu, WaitpidFailedSnafu,
};
use nix::fcntl::OFlag;
use nix::sys::ptrace;
use nix::sys::signal::{kill, Signal};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{execvp, fork, pipe2, ForkResult, Pid};
use snafu::ResultExt;
use std::ffi::CString;
use std::path::Path;
use std::process::exit;

/// Waits for a signal from the process with the given `pid`.
///
/// This function wraps `waitpid` to wait for the process state to change
/// and returns the current `WaitStatus` of the process.
///
/// # Returns
/// `Ok(WaitStatus::StillAlive)` if the process state changed successfully, or a `SdbError` if an error occurred.
///
/// # Errors
/// Returns an error if waiting on the process fails, wrapping the original `waitpid` error.
///
/// # Example
/// ```no_run
/// let pid = Pid::from_raw(12345);
/// let status = wait_on_signal(pid);
/// ```
pub fn wait_on_signal(pid: Pid) -> Result<WaitStatus> {
    waitpid(pid, None).context(WaitpidFailedSnafu)
}

/// A structure representing a managed process.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Process {
    /// Process ID
    pub pid: Pid,
    /// Whether the process should be terminated on drop
    terminate_on_end: bool,
    /// Current state of the process
    pub state: WaitStatus,
}

impl Process {
    /// Launches a new process from the specified executable path.
    ///
    /// This function forks the current process and attempts to execute the provided `path` in the
    /// child process. The parent process will wait for the child process to start and return
    /// a `Process` struct representing the launched process.
    ///
    /// # Errors
    /// Returns an error if the fork or exec fails, wrapping the underlying errors.
    ///
    /// # Panics
    /// Failed write parent pipe fd.
    ///
    /// # Example
    /// ```no_run
    /// let process = Process::launch(Path::new("/bin/ls"))?;
    /// ```
    pub fn launch(path: &Path, debug: bool) -> Result<Self> {
        let (read_fd, write_fd) = pipe2(OFlag::O_CLOEXEC).context(CouldNotCreatePipeSnafu)?;

        let pid = unsafe { fork() }
            .map_err(|err| SdbError::ForkFailed { source: err })
            .and_then(|result| match result {
                ForkResult::Parent { child } => Ok(child),
                ForkResult::Child => {
                    // Allow tracing of branched processes.
                    if let Err(err) = ptrace::traceme().context(TracingFailedSnafu){
                        err.write_to_fd(&write_fd)?;
                        exit(-1);
                     };
                    let c_string = CString::new(path.to_string_lossy().to_string()).map_err(|_| NullSnafu.build())?;
                    if let Err(e) = execvp(c_string.as_c_str(), &[c_string.as_c_str()]) {
                        let error = SdbError::ExecFailed { source: e };
                        error.write_to_fd(&write_fd)?;
                        exit(-1);
                    };
                    unreachable!("The forking process will be asked to execute the specified program and will not return here.")
                }
            })?;

        drop(write_fd); // When 1 byte is written or the `write` side is closed(drop), the wait for `read` is over.
        if let Ok(Some(err)) = SdbError::wait_read_from_fd(&read_fd) {
            let _ = wait_on_signal(pid); // wait child
            return Err(err);
        }

        Ok(Self {
            pid,
            terminate_on_end: true,
            state: {
                if debug {
                    wait_on_signal(pid)?
                } else {
                    WaitStatus::Stopped(pid, Signal::SIGSTOP)
                }
            },
        })
    }

    /// Attaches to an existing process with the given PID.
    ///
    /// This function uses `ptrace::attach` to attach to an existing process identified by `pid`.
    /// Once attached, it waits for the process state to change and returns a `Process` struct.
    ///
    /// # Errors
    /// Returns an error if attaching to the process fails, wrapping the underlying `ptrace` error.
    ///
    /// # Example
    /// ```no_run
    /// let process = Process::attach(12345)?;
    /// ```
    pub fn attach(pid: i32) -> Result<Self> {
        let pid = Pid::from_raw(pid);
        ptrace::attach(pid).context(CouldNotAttachSnafu)?;

        Ok(Self {
            pid,
            terminate_on_end: true,
            state: wait_on_signal(pid)?,
        })
    }

    /// Resumes execution of the attached process.
    ///
    /// This function uses `ptrace::cont` to continue the execution of the process
    /// and waits for the next state change.
    ///
    /// # Errors
    /// Returns an error if resuming the process fails, wrapping the underlying `ptrace` error.
    ///
    /// # Example
    /// ```no_run
    /// let mut process = Process::attach(12345)?;
    /// process.resume()?;
    /// ```
    pub fn resume(&mut self) -> Result<()> {
        ptrace::cont(self.pid, None).context(CouldNotResumeSnafu)?;
        self.state = wait_on_signal(self.pid)?;
        Ok(())
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        if self.pid.as_raw() != 0 {
            if self.state == WaitStatus::StillAlive {
                if let Err(_errno) = kill(self.pid, Signal::SIGSTOP) {
                    #[cfg(feature = "tracing")]
                    tracing::error!("failed kill with SIGSTOP: {_errno}")
                };
                let _ = waitpid(self.pid, None);
            }
            if let Err(_errno) = ptrace::detach(self.pid, None) {
                #[cfg(feature = "tracing")]
                tracing::error!("failed detach {_errno}")
            };
            if let Err(_errno) = kill(self.pid, Signal::SIGCONT) {
                #[cfg(feature = "tracing")]
                tracing::error!("failed kill with SIGCONT: {_errno}")
            };

            if self.terminate_on_end {
                if let Err(_errno) = kill(self.pid, Signal::SIGKILL) {
                    #[cfg(feature = "tracing")]
                    tracing::error!("failed kill with SIGKILL: {_errno}")
                };
                let _ = waitpid(self.pid, None);
            }
        }
    }
}
