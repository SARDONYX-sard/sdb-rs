//! `sdb`(lib) errors

/// Cli error
#[derive(Debug, snafu::Snafu)]
#[snafu(visibility(pub))]
pub enum SdbError {
    /// Fork failed: {source}
    ForkFailed { source: nix::errno::Errno },

    /// Tracing failed: {source}
    TracingFailed { source: nix::errno::Errno },

    /// Execute failed: {source}
    ExecFailed { source: nix::errno::Errno },

    /// `waitpid` failed: {source}
    WaitpidFailed { source: nix::errno::Errno },

    /// Could not resume: {source}
    CouldNotResume { source: nix::errno::Errno },

    /// Could not attach: {source}
    CouldNotAttach { source: nix::errno::Errno },

    /// Contain null bytes in a string error
    #[snafu(transparent)]
    NullError {
        /// Contain null bytes in a string error
        source: std::ffi::NulError,
        /// error location
        #[snafu(implicit)]
        location: snafu::Location,
    },

    /// Tracing log error
    #[cfg(feature = "tracing")]
    #[snafu(transparent)]
    TracingError {
        source: tracing::subscriber::SetGlobalDefaultError,
    },
}

/// `Result` for `sdb`(CLI) wrapper crate.
pub type Result<T, E = SdbError> = core::result::Result<T, E>;
