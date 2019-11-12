// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

extern crate android_logger;
extern crate jni;

use self::jni::objects::{JClass, JString};
use self::jni::JNIEnv;
use crate::tapyrus_spv_run;
use android_logger::{Config, FilterBuilder};
use log::Level;

/// Make it possible to show logs on android
#[no_mangle]
pub extern "system" fn Java_com_chaintope_tapyrus_spv_FFI_enableLog(_env: JNIEnv, _class: JClass) {
    android_logger::init_once(
        Config::default()
            .with_min_level(Level::Trace) // limit log level
            .with_tag("libtapyrus_spv")
            .with_filter(
                // configure messages for specific crate
                FilterBuilder::new()
                    .parse("error,tapyrus_spv=trace")
                    .build(),
            ),
    );
}

/// Run spv node
#[no_mangle]
pub unsafe extern "C" fn Java_com_chaintope_tapyrus_spv_FFI_spvRun(
    env: JNIEnv,
    _: JClass,
    remote: JString,
    network: JString,
    genesisHex: JString,
) {
    tapyrus_spv_run(
        env.get_string(remote)
            .expect("invalid pattern string")
            .as_ptr(),
        env.get_string(network)
            .expect("invalid pattern string")
            .as_ptr(),
        env.get_string(genesisHex)
            .expect("invalid pattern string")
            .as_ptr(),
    )
}
