use std::{
    net::SocketAddr,
    time::{SystemTime, UNIX_EPOCH},
    sync::atomic::{AtomicUsize, Ordering},
};
use rand::{RngCore, thread_rng};
use tokio::{
    prelude::*,
    net::TcpStream,
    codec::Framed,
};
use bitcoin::{
    network::{
        message_network::VersionMessage,
        message::NetworkMessage,
        address::Address,
    },
};
use crate::network::codec::NetworkMessagesCodec;
use crate::network::handle_connection::NetworkMessageStream;

pub struct Peer {
    pub id: u64,
    pub addr: SocketAddr,
    pub stream: NetworkMessageStream,
    pub version: Option<VersionMessage>,
}

impl Peer {
    pub fn new(id: u64, socket: TcpStream) -> Peer {
        let addr = socket.peer_addr().unwrap();
        let mut framed = Framed::new(socket, NetworkMessagesCodec::new());

        // start handshake
        let _ = framed.start_send(NetworkMessage::Version(version_message()));

        Peer {
            id,
            addr,
            stream: framed,
            version: None,
        }
    }
}

impl Future for Peer {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        trace!("Peer {} polling...", self.id);

        loop {
            match self.stream.poll().unwrap() {
                Async::Ready(Some(message)) => {
                    match message {
                        NetworkMessage::Version(version) => {
                            trace!("Receive version message: {:?}", version);
                            self.version = Some(version);

                            // send verack message
                            let _ = self.stream.start_send(NetworkMessage::Verack);
                        }
                        message => {
                            trace!("Receive message: {:?}", message);
                        }
                    }
                }
                Async::Ready(None) => unreachable!(),
                Async::NotReady => break,
            }
        }

        // flush all queued sending messages.
        let _ = self.stream.poll_complete();

        Ok(Async::NotReady)
    }
}

pub fn version_message() -> VersionMessage {
    let remote = "127.0.0.1:18444".parse().unwrap();
    let local = "127.0.0.1:18444".parse().unwrap();

    // now in unix time
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

    const SERVICE_BLOCKS:u64 = 1;
    const SERVICE_WITNESS:u64 =  1 << 3;

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