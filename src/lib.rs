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

use bitcoin::network::constants::Network;
use crate::network::connect;
use tokio::prelude::Future;

mod network;

#[cfg(test)]
mod test_helpers;


/// SPV
#[derive(Clone)]
pub struct SPV {
    network: Network,
}

impl SPV {
    /// returns SPV instance.
    pub fn new() -> SPV {
        SPV {
            network: Network::Testnet,
        }
    }

    /// run spv node.
    pub fn run(&self) {
        info!("start SPV node.");

        let connection = connect("127.0.0.1:18444")
            .map_err(|e| { eprintln!("{:?}", e) })
            .and_then(|peer| { peer });
        tokio::run(connection);
    }
}