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
const GENESIS_FOR_TEST: &str = "010000000000000000000000000000000000000000000000000000000000000000000000623fd6e71aaec98e129d8b447ba7c6fe88cd27346cc556353d2d8232a2829f0a49b4a19f4dc3f0526dca905dcaff6a8e34537d04b450e0ac5568ce89a9373e301665c860012102260b9be70a87125fd0e2da368db857a2d8ee1cb85a3c8b81490f4f35f99b212a40f457d5dd7caae6bf89a50efd13cf4a9e3857760f747e107d1184263e1212ef98cda52410bd822e92c44b4a22a2f6116a0df2b130afe315af6a7289567b32c1ca01010000000100000000000000000000000000000000000000000000000000000000000000000000000000ffffffff0100f2052a010000002776a9226d6b597162714c54584e52344568747853376e37734e5357385646546f314e55336e88ac00000000";

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
