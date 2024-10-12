//! `sdb`(lib) errors

use nix::errno::Errno;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::os::fd::{AsFd, AsRawFd, OwnedFd};

/// Custom serializer for `nix::errno::Errno`
fn serialize_errno<S>(errno: &Errno, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i32(*errno as i32)
}

/// Custom deserializer for `nix::errno::Errno`
fn deserialize_errno<'de, D>(deserializer: D) -> Result<Errno, D::Error>
where
    D: Deserializer<'de>,
{
    let code = i32::deserialize(deserializer)?;
    Ok(Errno::from_raw(code))
}

/// Custom error type for the `sdb` library.
#[derive(Debug, snafu::Snafu, Serialize, Deserialize)]
#[snafu(visibility(pub))]
pub enum SdbError {
    /// Failed to create pipe: {source}
    #[serde(
        serialize_with = "serialize_errno",
        deserialize_with = "deserialize_errno"
    )]
    CouldNotCreatePipe {
        source: Errno,
    },

    /// Fork failed: {source}
    #[serde(
        serialize_with = "serialize_errno",
        deserialize_with = "deserialize_errno"
    )]
    ForkFailed {
        source: Errno,
    },

    /// [Launch Error: Tracing failed]: {source}
    #[serde(
        serialize_with = "serialize_errno",
        deserialize_with = "deserialize_errno"
    )]
    TracingFailed {
        source: Errno,
    },

    /// [Launch Error: Execute failed] {source}
    #[serde(
        serialize_with = "serialize_errno",
        deserialize_with = "deserialize_errno"
    )]
    ExecFailed {
        source: Errno,
    },

    /// `waitpid` failed: {source}
    #[serde(
        serialize_with = "serialize_errno",
        deserialize_with = "deserialize_errno"
    )]
    WaitpidFailed {
        source: Errno,
    },

    /// Could not resume: {source}
    #[serde(
        serialize_with = "serialize_errno",
        deserialize_with = "deserialize_errno"
    )]
    CouldNotResume {
        source: Errno,
    },

    /// Could not attach: {source}
    #[serde(
        serialize_with = "serialize_errno",
        deserialize_with = "deserialize_errno"
    )]
    CouldNotAttach {
        source: Errno,
    },

    Null,

    /// Failed to serialize error
    SerializeErr {
        msg: String,
    },
    /// Failed to write fd.
    WriteFd,

    /// Failed to deserialize error
    DeserializeErr {
        msg: String,
    },
    /// Failed to read fd.
    ReadFd,
}

impl SdbError {
    /// Writes the `SdbError` instance to a file descriptor.
    ///
    /// This method serializes the `SdbError` enum using `bincode` and writes
    /// the serialized bytes to the provided `OwnedFd`.
    ///
    /// # Errors
    /// Returns an `io::Result<()>` if writing to the file descriptor fails.
    pub fn write_to_fd(&self, fd: impl AsFd) -> Result<()> {
        let encoded: Vec<u8> =
            bincode::serialize(self).map_err(|e| Self::SerializeErr { msg: e.to_string() })?;
        nix::unistd::write(fd, &encoded).map_err(|_| Self::WriteFd)?;
        Ok(())
    }

    /// Reads an `SdbError` instance from a file descriptor.
    ///
    /// This method reads bytes from the provided file descriptor, deserializes
    /// the bytes using `bincode`, and constructs an `SdbError` enum instance.
    ///
    /// # Errors
    /// Returns an `io::Result<SdbError>` if reading or deserialization fails.
    pub fn wait_read_from_fd(fd: &OwnedFd) -> Result<Option<Self>> {
        let mut buffer = [0; 1024]; // If vec is not cleared to 0, empty is always returned.
        if let Err(err) = nix::unistd::read(fd.as_raw_fd(), &mut buffer).map_err(|_| Self::ReadFd) {
            return Ok(Some(err));
        };

        // is_empty
        if buffer.iter().all(|&x| x == 0) {
            return Ok(None);
        }

        match bincode::deserialize(&buffer).map_err(|e| Self::DeserializeErr { msg: e.to_string() })
        {
            Err(err) | Ok(err) => Ok(Some(err)),
        }
    }
}

/// `Result` for `sdb`(CLI) wrapper crate.
pub type Result<T, E = SdbError> = core::result::Result<T, E>;
