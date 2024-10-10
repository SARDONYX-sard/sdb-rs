//! `sdb`(CLI) errors
use std::{io, path::PathBuf};

/// Cli error
#[allow(clippy::enum_variant_names)]
#[derive(Debug, snafu::Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    /// Failed I/O of {path}.
    #[snafu(display("{source}: {}", path.display()))]
    IoErrWithPath { source: io::Error, path: PathBuf },

    #[snafu(transparent)]
    ReadlineError {
        source: rustyline::error::ReadlineError,
    },

    #[snafu(transparent)]
    ClapError { source: clap::error::Error },

    #[snafu(transparent)]
    ErrNo { source: nix::errno::Errno },

    /// Tracing log error
    #[cfg(feature = "tracing")]
    #[snafu(transparent)]
    TracingError {
        source: tracing::subscriber::SetGlobalDefaultError,
    },
}

/// `Result` for `sdb`(CLI) wrapper crate.
pub type Result<T, E = Error> = core::result::Result<T, E>;
