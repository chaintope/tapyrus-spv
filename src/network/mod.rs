use std::{
    io,
    net::SocketAddr,
    time::{SystemTime, UNIX_EPOCH},
    sync::atomic::{AtomicUsize, Ordering},
};
use rand::{RngCore, thread_rng};
use tokio::{
    prelude::*,
    net::{
        TcpStream,
        tcp::ConnectFuture
    },
    codec::{
        Decoder,
        Encoder,
        Framed,
    },
    io::{
        Write,
        WriteHalf,
        ReadHalf,
    },
};
use bitcoin::{
    network::{
        message_network::VersionMessage,
        constants::Network,
        message::{NetworkMessage, RawNetworkMessage},
        address::Address,
        stream_reader::StreamReader,
    },
    consensus::Encodable,
};
use std::mem::transmute_copy;
use crate::network::codec::NetworkMessagesCodec;
use crate::network::handle_connection::NetworkMessageStream;

pub mod handle_connection;
pub mod bytes;
pub mod codec;


/// Interval for pinging peers
const PING_INTERVAL: u64 = 2 * 60;

pub struct Peer {
    id: u64,
    addr: SocketAddr,
    stream: NetworkMessageStream,
    version: Option<VersionMessage>,
}

impl Peer {
    fn new(id: u64, socket: TcpStream) -> Peer {
        let addr = socket.peer_addr().unwrap();

        let mut framed = Framed::new(socket, NetworkMessagesCodec::new());

        // start handshake
        let _ = framed.start_send(NetworkMessage::Version(version_message()));

//        let (sink, stream) = framed.split();
//
//        let send = sink.send(NetworkMessage::Version(version_message()))
//            .and_then(|sink| {
//                trace!("Send version message. {:?}", sink);
//                Ok(())
//            })
//            .map_err(|e| {
//                error!("Error: {}", e);
//            });
//        tokio::spawn(send);

        Peer {
            id,
            addr,
            stream: framed,
            version: None,
        }
    }

//    fn send(&mut self, message: NetworkMessage) -> impl Future<Item = NetworkMessageSink, Error = > {
//        trace!("Send message: peeer={}, message={:?}", self.id, message);
//        self.sink.send(message)
//    }

//    fn new(id: u64, address: &SocketAddr, socket: WriteHalf<TcpStream>) -> Peer {
//        Peer {
//            id,
//            address: address.clone(),
//            socket,
//            connected: false,
//            local_version: version_message(),
//            version: None,
//        }
//    }
//
//    pub fn send(& mut self, msg: NetworkMessage) -> Result<(), Error> {
//
//        trace!("Send message: peeer={}, message={:?}", self.id, msg);
//        let raw_msg = RawNetworkMessage {
//            magic: Network::Testnet.magic(),
//            payload: msg
//        };
//
//        let mut buffer = Vec::new();
//        raw_msg.consensus_encode(&mut buffer).unwrap();
//
//        self.socket.write(&buffer).unwrap();
//
//        Ok(())
//    }
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

                            let _ = self.stream.start_send(NetworkMessage::Verack);
                        }
                        message => {
                            trace!("Receive message: {:?}", message);
                        }
                    }
                }
                a => {
                    trace!("poll result: {:?}", a);
                    break;
                }
            }
        }

        // flush all queued message
        let _ = self.stream.poll_complete();


//        let send = self.stream.send(NetworkMessage::Version(version_message()))
//            .and_then(|sink| {
//                trace!("Send version message. {:?}", sink);
//                Ok(())
//            })
//            .map_err(|e| {
//                error!("Error: {}", e);
//            });
//        tokio::spawn(send);


//        match self.stream.poll() {
//            Ok(Async::Ready(Some(NetworkMessage::Version(version)))) => {
//                trace!("Receive version message: {:?}", version);
//                self.version = Some(version);
//
//                let _ = self.stream.start_send(NetworkMessage::Verack);
//            }
//            Ok(Async::Ready(Some(message))) => {
//                trace!("Receive message: {:?}", message);
//            }
//            Ok(Async::Ready(None)) => {
//                trace!("Ok(Async::Ready(None))");
//            }
//            Ok(Async::NotReady) => {
//                trace!("Ok(Async::NotReady)");
//            }
//            Err(e) => {
//                trace!("Error: {:?}", e);
//            }
//        }



//        match self.messages.poll() {
//            Ok(Async::Ready(Some(NetworkMessage::Version(version)))) => {
//                trace!("Receive version message: {:?}", version);
//                self.version = Some(version);
//            }
//            Ok(Async::Ready(Some(message))) => {
//                trace!("Receive message: {:?}", message);
//            }
//            Ok(Async::Ready(None)) => {
//                trace!("Ok(Async::Ready(None))");
//            }
//            Ok(Async::NotReady) => {
//                trace!("Ok(Async::NotReady)");
//            }
//            Err(e) => {
//                trace!("Error: {:?}", e);
//            }
//        }

//        self.messages.for_each(|message| {
//            match message {
//                NetworkMessage::Version(version) => {
//                    trace!("Receive version message: {:?}", version);
//                    self.version = Some(version);
//                }
//                m => { trace!("Receive message: {:?}", m) }
//            }
//        });
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

enum Error {
    IoError(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error { Error::IoError(e) }
}