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

use crate::chain::{Chain, OnMemoryChainStore};
use crate::network::{connect, BlockHeaderDownload, Handshake};
use bitcoin::network::constants::Network;
use std::sync::{Arc, Mutex};
use tokio::prelude::Future;

mod chain;
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

/// Manage blockchain status
pub struct ChainState {
    chain_active: Chain<OnMemoryChainStore>,
}

impl ChainState {
    /// create ChainState instance
    pub fn new() -> ChainState {
        ChainState {
            chain_active: Chain::<OnMemoryChainStore>::default(),
        }
    }

    /// borrow chain_active
    pub fn borrow_chain_active(&self) -> &Chain<OnMemoryChainStore> {
        &self.chain_active
    }

    /// borrow mutable chain_active
    pub fn borrow_mut_chain_active(&mut self) -> &mut Chain<OnMemoryChainStore> {
        &mut self.chain_active
    }
}
