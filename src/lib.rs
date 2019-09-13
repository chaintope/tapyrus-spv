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
use std::sync::{Arc, Mutex};
use crate::chain::ChainState;

mod network;
mod chain;

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

        let chain_state = Arc::new(Mutex::new(ChainState::new()));

        let chain_state_for_block_header_download = chain_state.clone();

        let connection = connect("127.0.0.1:18444", self.network)
            .and_then(|peer| Handshake::new(peer))
            .and_then(move |peer| {
                BlockHeaderDownload::new(peer, chain_state_for_block_header_download)
            })
            .map(move |_peer| {
                let chain_state = chain_state.lock().unwrap();
                let chain_active = chain_state.borrow_chain_active();
                info!("current block height: {}", chain_active.height());
            })
            .map_err(|e| error!("Error: {:?}", e));
        tokio::run(connection);
    }
}