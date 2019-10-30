// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

//! `network` module
//!
//! The `network` module contains p2p communication functionality.

mod peer;
pub use self::peer::connect;
pub use self::peer::Peer;

mod handshake;
pub use self::handshake::Handshake;

mod block_header_download;
pub use self::block_header_download::BlockHeaderDownload;

pub mod utils;

mod error;
pub use self::error::Error;
pub use self::error::MaliciousPeerCause;
