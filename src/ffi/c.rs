// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

use std::os::raw::{c_char};
use std::ffi::{CString, CStr};
use crate::{SPV, Options, ChainParams};
use bitcoin::Network;
use env_logger::Env;

/// initialize logger
#[no_mangle]
pub extern fn enable_log() {
    let env = Env::new()
        .filter("RUST_LOG")
        .write_style("error,tapyrus_spv=trace");

    env_logger::try_init_from_env(env).unwrap();
}

#[no_mangle]
pub extern fn rust_greeting(to: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(to) };
    let recipient = match c_str.to_str() {
        Err(_) => "there",
        Ok(string) => string,
    };

    let params = Options {
        remote: "192.168.0.44:18444".to_string(),
        datadir: "/tmp/tapyrus-spv".to_string(),
        chain_params: ChainParams {
            network: Network::Regtest,
        },
    };

    let spv = SPV::new(params);
     spv.run();

    CString::new("aaHello ".to_owned() + recipient + "from rust").unwrap().into_raw()
}

#[no_mangle]
pub extern fn rust_greeting_free(s: *mut c_char) {
    unsafe {
        if s.is_null() { return }
        CString::from_raw(s)
    };
}
