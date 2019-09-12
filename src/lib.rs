//! # Tapyrus SPV Library
//!
//!
//!

#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(missing_docs)]
#![deny(unused_must_use)]
#![forbid(unsafe_code)]
#![feature(async_await)]

extern crate bitcoin;
extern crate tokio;
#[macro_use]
extern crate log;
extern crate bytes;

use crate::network::{connect, Handshake, BlockHeaderDownload};
use bitcoin::network::constants::Network;
use tokio::prelude::Future;
use bitcoin::blockdata::constants::genesis_block;
use bitcoin::BlockHeader;
use std::sync::{Arc, Mutex};

mod network;

#[cfg(test)]
mod test_helper;

/// SPV
#[derive(Clone)]
pub struct SPV {
    network: Network,
}

impl SPV {
    /// returns SPV instance.
    pub fn new(network: Network) -> SPV {
        SPV { network }
    }

    /// run spv node.
    pub fn run(&self) {
        info!("start SPV node.");

        let genesis = genesis_block(Network::Regtest);
        let headers = Arc::new(Mutex::new(vec![genesis.header]));

        let headers_for_block_header_download = headers.clone();

        let connection = connect("127.0.0.1:18444", self.network)
            .and_then(|peer| Handshake::new(peer))
            .and_then(move |peer| {
                BlockHeaderDownload::new(peer, headers_for_block_header_download)
            })
            .map(move |_peer| {
                let headers = headers.lock().unwrap();
                info!("block count: {}", headers.len());
            })
            .map_err(|e| error!("Error: {:?}", e));
        tokio::run(connection);
    }
}