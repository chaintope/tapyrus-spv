use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicUsize, Ordering};
use bitcoin::{
    network::{
        message_network::VersionMessage,
        address::Address,
    }
};
use rand::{thread_rng, RngCore};

pub const SERVICE_BLOCKS:u64 = 1;
pub const SERVICE_WITNESS:u64 =  1 << 3;

pub fn version_message() -> VersionMessage {
    let remote = "127.0.0.1:18444".parse().unwrap();
    let local = "127.0.0.1:18444".parse().unwrap();

    // now in unix time
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

    let services = SERVICE_BLOCKS + SERVICE_WITNESS;
    let mut rng =  thread_rng();
    let nonce = rng.next_u64(); // クライアントごとに固定？

    let height = AtomicUsize::new(0);

    // build message
    VersionMessage::new(
        services,
        timestamp,
        Address::new(&remote, 1),
        Address::new(&local, 1),
        nonce,
        "/bitcoin-spv:0.1.0/".to_string(),
        height.load(Ordering::Relaxed) as i32
    )
}