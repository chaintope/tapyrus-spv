// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

use crate::chain;
use crate::network::peer::PeerID;
use crate::network::utils::codec;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    CodecError(codec::Error),
    UnboundedSendError(tokio::sync::mpsc::error::UnboundedSendError),
    UnboundedRecvError(tokio::sync::mpsc::error::UnboundedRecvError),
    MaliciousPeer(PeerID, MaliciousPeerCause),
    ChainError(chain::Error),
    WrongMagicBytes,
}

#[derive(Debug)]
pub enum MaliciousPeerCause {
    /// The peer send over maximum number which is MAX_HEADERS_RESULTS of headers in single
    /// headers message.
    SendOverMaxHeadersResults,
    /// The peer send non-continuous headers sequence.
    SendNonContinuousHeadersSequence,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IoError(e)
    }
}

impl From<codec::Error> for Error {
    fn from(e: codec::Error) -> Error {
        Error::CodecError(e)
    }
}

impl From<tokio::sync::mpsc::error::UnboundedSendError> for Error {
    fn from(e: tokio::sync::mpsc::error::UnboundedSendError) -> Error {
        Error::UnboundedSendError(e)
    }
}

impl From<tokio::sync::mpsc::error::UnboundedRecvError> for Error {
    fn from(e: tokio::sync::mpsc::error::UnboundedRecvError) -> Error {
        Error::UnboundedRecvError(e)
    }
}
