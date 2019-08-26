//! `network` module
//!
//! The `network` module contains p2p communication functionality.

mod peer;
pub use peer::Peer;

mod handle_connection;
pub use handle_connection::*;

pub mod bytes;
pub mod codec;

enum Error {
    IoError(std::io::Error),
    WrongMagicBytes,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IoError(e)
    }
}
