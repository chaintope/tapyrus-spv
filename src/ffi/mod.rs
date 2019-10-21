// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

pub mod c;

#[cfg(target_os="android")]
#[allow(non_snake_case)]
pub mod android;
