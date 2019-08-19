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

pub mod handle_connection;
pub mod bytes;
pub mod codec;


/// Interval for pinging peers
const PING_INTERVAL: u64 = 2 * 60;

#[derive(Debug)]
struct Peer {
    id: u64,
    address: SocketAddr,
    socket: WriteHalf<TcpStream>,
    /// whether handshake is done.
    connected: bool,
    local_version: VersionMessage,
    version: Option<VersionMessage>,
}

impl Peer {
    fn new(id: u64, address: &SocketAddr, socket: WriteHalf<TcpStream>) -> Peer {
        Peer {
            id,
            address: address.clone(),
            socket,
            connected: false,
            local_version: version_message(),
            version: None,
        }
    }

    fn send(& mut self, msg: NetworkMessage) -> Result<(), Error> {
        trace!("Send message: peeer={}, message={:?}", self.id, msg);
        let raw_msg = RawNetworkMessage {
            magic: Network::Testnet.magic(),
            payload: msg
        };

        let mut buffer = Vec::new();
        raw_msg.consensus_encode(&mut buffer).unwrap();

        self.socket.write(&buffer).unwrap();

        Ok(())
    }
}

fn version_message() -> VersionMessage {
    unimplemented!();
}

enum Error {
    IoError(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error { Error::IoError(e) }
}