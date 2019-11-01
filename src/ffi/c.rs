// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

use crate::{ChainParams, Options, SPV};
use env_logger::Env;
use std::ffi::CStr;
use std::os::raw::c_char;
use tapyrus::consensus::deserialize;
use tapyrus::Network;

/// initialize logger
#[no_mangle]
pub extern "C" fn tapyrus_enable_log() {
    let env = Env::new()
        .filter("RUST_LOG")
        .write_style("error,tapyrus_spv=trace");

    env_logger::try_init_from_env(env).unwrap();
}

/// run spv
#[no_mangle]
pub extern "C" fn tapyrus_spv_run(
    remote: *const c_char,
    network: *const c_char,
    genesis_hex: *const c_char,
) {
    let remote = unsafe { CStr::from_ptr(remote) }
        .to_str()
        .expect("wrong string passed as remote address.")
        .to_string();

    let network = unsafe { CStr::from_ptr(network) }
        .to_str()
        .expect("wrong string passed as network.");

    let network = match network {
        "bitcoin" => Network::Bitcoin,
        "testnet" => Network::Testnet,
        "regtest" => Network::Regtest,
        _ => panic!("network should be \"bitcoin\" or \"testnet\" or \"regtest\""),
    };

    let genesis_hex = unsafe { CStr::from_ptr(genesis_hex) }
        .to_str()
        .expect("wrong string passed as genesis_hex.");

    let genesis = deserialize(&hex::decode(genesis_hex).expect("genesis_hex is invalid hex."))
        .expect("genesis_hex is invalid block data");

    let params = Options {
        remote,
        datadir: "/tmp/tapyrus-spv".to_string(),
        chain_params: ChainParams { network, genesis },
    };

    let spv = SPV::new(params);
    spv.run();
}
