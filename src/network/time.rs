// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

use core::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::SystemTime;

static MOCK_TIME: AtomicU64 = AtomicU64::new(0);

/// returns adjusted time with other nodes.
/// TODO: Implement actual logic refer to https://github.com/chaintope/tapyrus-core/blob/eb7daf4d600eeb631427c018a984a77a34aca66e/src/timedata.cpp#L35
pub fn get_adjusted_time() -> u64 {
    now()
}

/// returns current unix time
pub fn now() -> u64 {
    let mock = MOCK_TIME.load(Ordering::Relaxed);
    if mock != 0 {
        return mock;
    }
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    assert!(now > 0);
    now
}

pub fn set_mock_time(mock_time_in: u64) {
    MOCK_TIME.store(mock_time_in, Ordering::Relaxed);
}

pub fn get_mock_time() -> u64 {
    MOCK_TIME.load(Ordering::Relaxed)
}
