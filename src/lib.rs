// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

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

extern crate tapyrus;
extern crate tokio;
#[macro_use]
extern crate log;
extern crate byteorder;
extern crate bytes;

use crate::chain::store::OnMemoryChainStore;
use crate::chain::{Chain, ChainStore};
use crate::network::{connect, BlockHeaderDownload, Handshake};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tapyrus::network::constants::Network;
use tapyrus::Block;
use tokio::prelude::Future;

mod chain;
mod ffi;
mod network;

#[cfg(target_os = "android")]
pub use crate::ffi::android::*;
pub use crate::ffi::c::*;

#[cfg(test)]
mod test_helper;

/// SPV
#[derive(Clone)]
pub struct SPV {
    options: Options,
}

impl SPV {
    /// returns SPV instance.
    pub fn new(params: Options) -> SPV {
        SPV { options: params }
    }

    /// run spv node.
    pub fn run(&self) {
        info!("Start SPV node.");

        // initialize chain_state
        let datadir_path = Path::new(&self.options.datadir);
        info!("datadir is {}", datadir_path.display());
        let remote_socket_addr = self.options.remote.parse().expect(&format!(
            "Can not parse remote peer address: \"{}\"",
            self.options.remote
        ));

        let mut chain_store = OnMemoryChainStore::new();
        chain_store.initialize(self.options.chain_params.genesis.clone());
        let chain_active = Chain::new(chain_store);
        let chain_state = Arc::new(Mutex::new(ChainState::new(chain_active)));

        let chain_state_for_block_header_download = chain_state.clone();

        info!(
            "Connect to remote peer {}. Network is {}.",
            remote_socket_addr, self.options.chain_params.network
        );
        let connection = connect(&remote_socket_addr, self.options.chain_params.network)
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
pub struct ChainState<T: ChainStore> {
    chain_active: Chain<T>,
}

impl ChainState<OnMemoryChainStore> {
    /// create ChainState instance
    pub fn new<T: ChainStore>(chain_active: Chain<T>) -> ChainState<T> {
        ChainState { chain_active }
    }
}

impl<T: ChainStore> ChainState<T> {
    /// borrow chain_active
    pub fn borrow_chain_active(&self) -> &Chain<T> {
        &self.chain_active
    }

    /// borrow mutable chain_active
    pub fn borrow_mut_chain_active(&mut self) -> &mut Chain<T> {
        &mut self.chain_active
    }
}

/// Parameters for SPV node
#[derive(Debug, Clone)]
pub struct Options {
    /// Remote peer address to connect.
    pub remote: String,
    /// Data directory for putting database files.
    pub datadir: String,
    /// Chain parameter for network type which the SPV node work on.
    pub chain_params: ChainParams,
}

/// Parameters for Blockchain network
#[derive(Debug, Clone)]
pub struct ChainParams {
    /// Network Type
    pub network: Network,
    /// Genesis block for network to be connected
    pub genesis: Block,
}
