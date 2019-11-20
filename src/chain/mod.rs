// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

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
    EncodeError(tapyrus::consensus::encode::Error),
    BitcoinHashesError(bitcoin_hashes::Error),
    /// The error which arises under validating blocks.
    BlockValidationError(BlockValidationErrorCause),
}

/// Causes of BlockValidationError
#[derive(Debug)]
pub enum BlockValidationErrorCause {
    /// The block can't connect current chain tip.
    CantConnectToTip,
    WrongBlockVersion {
        wrong_version: u32,
        correct_version: u32,
    },
    BlockTimeTooOld,
    BlockTimeTooNew,
}

impl From<tapyrus::consensus::encode::Error> for Error {
    fn from(e: tapyrus::consensus::encode::Error) -> Error {
        Error::EncodeError(e)
    }
}

impl From<bitcoin_hashes::Error> for Error {
    fn from(e: bitcoin_hashes::Error) -> Error {
        Error::BitcoinHashesError(e)
    }
}
