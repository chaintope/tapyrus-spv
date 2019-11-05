// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

#![forbid(unsafe_code)]
//! # Chain module
//!
//! This is a module for storing chains which is consisted of block headers and provide useful API
//! to access block headers in the chain.

mod block_index;
mod chain;
pub mod store;

pub use block_index::BlockIndex;
pub use chain::Chain;
pub use chain::ChainStore;

#[derive(Debug)]
pub enum Error {
    EncodeError(bitcoin::consensus::encode::Error),
    BitcoinHashesError(bitcoin_hashes::Error),
}

impl From<bitcoin::consensus::encode::Error> for Error {
    fn from(e: bitcoin::consensus::encode::Error) -> Error {
        Error::EncodeError(e)
    }
}

impl From<bitcoin_hashes::Error> for Error {
    fn from(e: bitcoin_hashes::Error) -> Error {
        Error::BitcoinHashesError(e)
    }
}
