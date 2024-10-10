//! `sdb`(lib) errors

/// Cli error
#[allow(clippy::enum_variant_names)]
#[derive(Debug, snafu::Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    /// Tracing log error
    #[cfg(feature = "tracing")]
    #[snafu(transparent)]
    TracingError {
        source: tracing::subscriber::SetGlobalDefaultError,
    },
}

/// `Result` for `sdb`(CLI) wrapper crate.
pub type Result<T, E = Error> = core::result::Result<T, E>;
