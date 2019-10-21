// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

extern crate jni;

use super::*;
use self::jni::JNIEnv;
use self::jni::objects::{JClass, JString};
use self::jni::sys::{jstring};
use crate::ffi::c::rust_greeting;
use std::ffi::CString;

#[no_mangle]
pub unsafe extern fn Java_com_chaintope_tapyrus_spv_RustGreetings_greeting(env: JNIEnv, _: JClass, java_pattern: JString) -> jstring {
    // Our Java companion code might pass-in "world" as a string, hence the name.
    let world = rust_greeting(env.get_string(java_pattern).expect("invalid pattern string").as_ptr());
    // Retake pointer so that we can use it below and allow memory to be freed when it goes out of scope.
    let world_ptr = CString::from_raw(world);
    let output = env.new_string(world_ptr.to_str().unwrap()).expect("Couldn't create java string!");

    output.into_inner()
}