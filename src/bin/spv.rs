// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.
extern crate tapyrus_spv;

extern crate log;

use tapyrus::consensus::deserialize;
use tapyrus::network::constants::Network;
use tapyrus_spv::{ChainParams, Options, SPV};

/// This Genesis Block HEX is for test.
///
/// You should set regtest mode when using this genesis block.
///
/// The aggregated keys for the chain based on this genesis block is here.
/// private key: 9b90c1704259341b5d08a585abe3544f8b4a10dfdc97b402d274220c06da28a2
/// public key: 02260b9be70a87125fd0e2da368db857a2d8ee1cb85a3c8b81490f4f35f99b212a
const GENESIS_FOR_TEST: &str = "01000000000000000000000000000000000000000000000000000000000000000000000019225b47d8c3eefd0dab934ba4b633940d032a7f5a192a4ddece08f566f1cfb95d5022ed80bde51d7436cadcb10455a2e5523fea9e46dc9ee5dec0037387e1b137aaba5d40fd3748264662cd991ac70e8d9ae3e06a1ea8956d74b36aa6419ca428f9baf24dc4df95637dc524d6374ef59ef6d15aba25020de3c35da969b1329ec961488067010000002001000000000000000000000000000000000000000000000000000000000000000000000000222102260b9be70a87125fd0e2da368db857a2d8ee1cb85a3c8b81490f4f35f99b212affffffff0100f2052a010000001976a9143733df9979ee67615b16aff5b210d894557325df88ac00000000";

fn main() {
    env_logger::init();

    let params = Options {
        remote: "127.0.0.1:12383".to_string(),
        datadir: "/tmp/tapyrus-spv".to_string(),
        chain_params: ChainParams {
            network: Network::Dev,
            genesis: deserialize(&hex::decode(GENESIS_FOR_TEST).unwrap()).unwrap(),
            network_id: tapyrus::network::constants::NetworkId::REGTEST,
        },
    };

    let spv = SPV::new(params);
    spv.run();
}
