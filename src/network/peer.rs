use std::{
    net::SocketAddr,
    time::{SystemTime, UNIX_EPOCH},
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
use std::borrow::BorrowMut;

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
        trace!("Sending message: {:?}", message);

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
    let blank_addr = "[0:0:0:0:0:0:0:0]:0".parse().unwrap();

    // now in unix time
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

    let services = 0;

    // generate random value
    let nonce =  thread_rng().borrow_mut().next_u64();

    // TODO: after block database is constructed, set actual latest block height.
    let start_height = 0;

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // build message
    VersionMessage::new(
        services,
        timestamp,
        Address::new(&blank_addr, 0),
        Address::new(&blank_addr, services),
        nonce,
        format!("/bitcoin-spv:{}/", VERSION),
        start_height
    )
}