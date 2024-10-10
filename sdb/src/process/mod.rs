use nix::errno::Errno;
use nix::sys::ptrace;
use nix::sys::signal::{kill, Signal};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{execvp, fork, getpid, ForkResult, Pid};
use std::ffi::CString;
use std::path::Path;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Process {
    pub pid: Pid,
    terminate_on_end: bool,
    pub state: WaitStatus,
}

impl Process {
    pub fn launch(path: &Path) -> Result<Self, Errno> {
        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                waitpid(child, None)?;
            }
            Ok(ForkResult::Child) => {
                let c_string = CString::new(path.to_string_lossy().to_string()).unwrap();
                execvp(c_string.as_c_str(), &[c_string.as_c_str()])?;
            }
            Err(err) => return Err(err),
        }

        ptrace::traceme()?;
        let pid = getpid();
        Ok(Self {
            pid,
            terminate_on_end: true,
            state: waitpid(pid, None)?,
        })
    }

    pub fn attach(pid: i32) -> Result<Self, Errno> {
        // if pid == 0 {
        //     return Err("Invalid PID".to_string());
        // }
        let pid = Pid::from_raw(pid);
        ptrace::attach(pid)?;

        Ok(Self {
            pid,
            terminate_on_end: true,
            state: waitpid(pid, None)?,
        })
    }

    pub fn resume(&mut self) -> Result<(), Errno> {
        ptrace::cont(self.pid, None)?;
        self.state = self.wait_on_signal()?;
        Ok(())
    }

    pub fn wait_on_signal(&self) -> Result<WaitStatus, Errno> {
        waitpid(self.pid, None)
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        if self.pid.as_raw() != 0 {
            if self.state == WaitStatus::StillAlive {
                let _ = kill(self.pid, Signal::SIGSTOP);
                let _ = waitpid(self.pid, None);
            }
            let _ = ptrace::detach(self.pid, None);
            let _ = kill(self.pid, Signal::SIGCONT);

            if self.terminate_on_end {
                let _ = kill(self.pid, Signal::SIGKILL);
                let _ = waitpid(self.pid, None);
            }
        }
    }
}
