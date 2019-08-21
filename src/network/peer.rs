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
        message::{
            RawNetworkMessage,
            NetworkMessage,
        },
        address::Address,
        constants::Network,
    },
};
use crate::network::codec::NetworkMessagesCodec;
use crate::network::handle_connection::NetworkMessageStream;
use crate::network::Error;

pub struct Peer {
    pub id: u64,
    pub addr: SocketAddr,
    pub network: Network,
    pub stream: NetworkMessageStream,
    pub version: Option<VersionMessage>,
}

impl Peer {
    pub fn new(id: u64, socket: TcpStream, network: Network) -> Peer {
        let addr = socket.peer_addr().unwrap();
        let framed = Framed::new(socket, NetworkMessagesCodec::new());

        let mut peer = Peer {
            id,
            addr,
            network,
            stream: framed,
            version: None,
        };
        peer.start_handshake();

        peer
    }

    pub fn start_handshake(&mut self) {
        // start handshake
        let _ = self.start_send(NetworkMessage::Version(version_message()));
    }

    pub fn start_send(&mut self, message: NetworkMessage) {
        let raw_msg = RawNetworkMessage {
            magic: self.network.magic(),
            payload: message
        };

        let _ = self.stream.start_send(raw_msg);
    }

    fn process_message(&mut self, message: RawNetworkMessage) -> Result<(), Error> {
        if message.magic != self.network.magic() {
            info!("Wrong magic bytes.");
            return Err(Error::WrongMagicBytes);
        }

        match message.payload {
            NetworkMessage::Version(version) => {
                trace!("Receive version message: {:?}", version);
                self.version = Some(version);

                // send verack message
                let _ = self.start_send(NetworkMessage::Verack);
            }
            message => {
                trace!("Receive message: {:?}", message);
            }
        }

        Ok(())
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
                    if let Err(Error::WrongMagicBytes) = self.process_message(message) {
                        // Stop polling this peer future by returning error, then tcp will be disconnected.
                        return Err(());
                    }
                }
                Async::Ready(None) => {},
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