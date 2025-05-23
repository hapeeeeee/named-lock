use thiserror::Error;

/// Type alias to `Result<T, Error>`.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Error type of this crate.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid character in name")]
    InvalidCharacter,

    #[cfg(not(target_arch = "wasm32"))]
    #[error("Failed to create named lock: {0}")]
    CreateFailed(#[source] std::io::Error),
    #[cfg(target_arch = "wasm32")]
    #[error("Failed to create named lock")]
    CreateFailed,

    #[error("Failed to lock named lock")]
    LockFailed,

    #[error("Failed to unlock named lock")]
    UnlockFailed,

    #[error("Named lock would block")]
    WouldBlock,
}
