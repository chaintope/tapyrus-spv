// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.
extern crate tapyrus_spv;

extern crate log;

use tapyrus::network::constants::Network;
use tapyrus_spv::{ChainParams, Options, SPV};

fn main() {
    env_logger::init();

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
