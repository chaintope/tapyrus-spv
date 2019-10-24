// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

extern crate jni;
extern crate android_logger;

use super::*;
use self::jni::JNIEnv;
use self::jni::objects::{JClass, JString};
use self::jni::sys::{jstring};
use crate::ffi::c::rust_greeting;
use bitcoin::Network;
use std::ffi::CString;
use log::Level;
use crate::{Options, ChainParams, SPV};
use android_logger::{Config, FilterBuilder};

#[no_mangle]
pub extern "system" fn Java_com_chaintope_tapyrus_spv_FFI_enableLog(
    env: JNIEnv,
    _class: JClass
) {
    android_logger::init_once(
        Config::default()
            .with_min_level(Level::Trace) // limit log level
            .with_tag("libtapyrus_spv")
            .with_filter( // configure messages for specific crate
                          FilterBuilder::new()
                              .parse("error,tapyrus_spv=trace")
                              .build())
    );
}

#[no_mangle]
pub unsafe extern fn Java_com_chaintope_tapyrus_spv_RustGreetings_greeting(env: JNIEnv, _: JClass, java_pattern: JString) -> jstring {
    // Our Java companion code might pass-in "world" as a string, hence the name.
    let world = rust_greeting(env.get_string(java_pattern).expect("invalid pattern string").as_ptr());
    // Retake pointer so that we can use it below and allow memory to be freed when it goes out of scope.
    let world_ptr = CString::from_raw(world);
    let output = env.new_string(world_ptr.to_str().unwrap()).expect("Couldn't create java string!");

    let params = Options {
        remote: "192.168.0.44:18444".to_string(),
        datadir: "/tmp/tapyrus-spv".to_string(),
        chain_params: ChainParams {
            network: Network::Regtest,
        },
    };

    let spv = SPV::new(params);
    spv.run();

    output.into_inner()
}