use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::{
    prelude::*,
    net::TcpListener,
    io::ErrorKind,
};
use bitcoin::{
    consensus::Encodable,
    network::{
        message::{
            NetworkMessage,
            RawNetworkMessage,
        },
        constants::Network,
        message_network::VersionMessage,
        address::Address,
    }
};
use rand::{thread_rng, RngCore};
use crate::network::handle_connection::connect;

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

pub fn run_dummy_remote_peer(addr: &str) -> impl Future<Item = (), Error=()>{
    let addr = addr.parse().unwrap();
    // create dummy server
    let listener = TcpListener::bind(&addr)
        .expect("unable to bind TCP listener");

    let incoming = listener.incoming();

    incoming
        .map_err(|e| eprintln!("accept failed = {:?}", e))
        .for_each(|socket| {
            trace!("incoming new connection, {:?}", socket);
            let (reader, writer) = socket.split();

            // handle reader
            let buf = vec!(0u8; 8);
            let handle_read = tokio::io::read_exact(reader, buf).map(|(reader, buf)| {
                trace!("Remote: read from buffer {:?}", buf);
            }).map_err(|err| {
                eprintln!("Remote: I/O error {:?}", err)
            });

            tokio::spawn(handle_read);

            // send version message
            let raw_msg = RawNetworkMessage {
                magic: Network::Testnet.magic(),
                payload: NetworkMessage::Version(version_message()),
            };

            let mut buffer = Vec::new();
            raw_msg.consensus_encode(&mut buffer).unwrap();

            let bytes_written = tokio::io::write_all(writer, buffer);
            let handle_conn = bytes_written.map(|amt| {
                trace!("Remote: wrote {:?} bytes", amt)
            }).map_err(|err| {
                eprintln!("I/O error {:?}", err)
            });

            tokio::spawn(handle_conn)
        })
}

pub fn create_client(addr: &str) -> impl tokio::prelude::Future<Item=(), Error=()> {
    connect(addr).and_then(|framed| {
        trace!("Local: Connect");

        let (sink, stream) = framed.split();
        let send = sink.send(NetworkMessage::Version(version_message()))
            .map_err(|e| println!("error = {:?}", e))
            .and_then(|a| {
                Ok(())
            });

        tokio::spawn(send);

        stream.for_each(|msg| {
            trace!("Local Receve {:?}", msg);
            Ok(())
        })
    }).map_err(|e| println!("error = {:?}", e))
}