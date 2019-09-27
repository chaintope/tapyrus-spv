extern crate simple_logger;

use bitcoin::network::constants::Network;
use log::Level;
use std::borrow::Borrow;
use tapyrus_spv::{ChainParams, Options, SPV};

fn main() {
    // TODO: specify log level by user argument
    simple_logger::init_with_level(Level::Trace).unwrap();

    let params = Options {
        remote: "127.0.0.1:18444".to_string(),
        datadir: "/tmp/tapyrus-spv".to_string(),
        chain_params: ChainParams {
            network: Network::Regtest,
        },
    };

    let spv = SPV::new(params);
    spv.run();
}
