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

use crate::network::{connect, Handshake};
use bitcoin::network::constants::Network;
use tokio::prelude::Future;

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

        let connection = connect("127.0.0.1:18444", self.network)
            .and_then(|peer| Handshake::new(peer))
            .map(|_peer| {})
            .map_err(|e| error!("Error: {:?}", e));
        tokio::run(connection);
    }
}
