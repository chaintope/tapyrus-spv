// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

use crate::chain::{Chain, ChainStore};
use crate::network::{utils::codec::NetworkMessagesCodec, Error};
use rand::{thread_rng, RngCore};
use std::{
    borrow::BorrowMut,
    net::SocketAddr,
    time::{SystemTime, UNIX_EPOCH},
};
use tapyrus::network::message_blockdata::GetHeadersMessage;
use tapyrus::network::{
    address::Address,
    constants::ServiceFlags,
    message::{NetworkMessage, RawNetworkMessage},
    message_network::VersionMessage,
};
use tapyrus::BlockHash;
use tokio::{codec::Framed, net::TcpStream, prelude::*};

pub type PeerID = u64;

pub struct Peer<T>
where
    T: Sink<SinkItem = RawNetworkMessage> + Stream<Item = RawNetworkMessage>,
{
    pub id: PeerID,
    pub addr: SocketAddr,
    pub magic: u32,
    pub stream: T,
    pub version: Option<VersionMessage>,
}

impl<T> Peer<T>
where
    T: Sink<SinkItem = RawNetworkMessage> + Stream<Item = RawNetworkMessage>,
{
    pub fn new(id: u64, stream: T, addr: SocketAddr, magic: u32) -> Peer<T> {
        Peer {
            id,
            addr,
            magic,
            stream,
            version: None,
        }
    }

    /// Start to send message.
    /// This function just put message into buffer on sink. So call stream.poll_complete() to  send
    /// to remote.
    pub fn start_send(&mut self, message: NetworkMessage) {
        trace!("Sending message: {:?}", message);

        let raw_msg = RawNetworkMessage {
            magic: self.magic,
            payload: message,
        };

        let _ = self.stream.start_send(raw_msg);
    }

    /// flush all queued sending messages.
    pub fn flush(&mut self) {
        let _ = self.stream.poll_complete();
    }

    /// Send getheaders message to peer.
    pub fn send_getheaders<S: ChainStore>(&mut self, chain: &Chain<S>) {
        let locators = chain.get_locator();
        let stop_hash = BlockHash::default();
        let getheaders = GetHeadersMessage::new(locators, stop_hash);
        self.start_send(NetworkMessage::GetHeaders(getheaders));
    }
}

impl<T> Stream for Peer<T>
where
    T: Sink<SinkItem = RawNetworkMessage> + Stream<Item = RawNetworkMessage>,
    Error: From<T::Error>,
{
    type Item = NetworkMessage;
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        match self.stream.poll()? {
            Async::Ready(Some(message)) => {
                if message.magic != self.magic {
                    info!("Wrong magic bytes.");
                    return Err(Error::WrongMagicBytes);
                }

                trace!("Receive message: {:?}", message);
                Ok(Async::Ready(Some(message.payload)))
            }
            Async::Ready(None) => Ok(Async::Ready(None)),
            Async::NotReady => Ok(Async::NotReady),
        }
    }
}

pub fn connect(
    address: &SocketAddr,
    magic: u32,
) -> impl Future<Item = Peer<Framed<TcpStream, NetworkMessagesCodec>>, Error = Error> {
    trace!("Try to create TCP connection to {}", address);
    TcpStream::connect(address)
        .map(move |stream| {
            let addr = stream.peer_addr().unwrap();
            trace!("Success to create TCP connection to {}", addr);
            let stream = Framed::new(stream, NetworkMessagesCodec::new());
            Peer::new(0, stream, addr, magic)
        })
        .map_err(|e| Error::from(e))
}

pub fn version_message() -> VersionMessage {
    let blank_addr = "[0:0:0:0:0:0:0:0]:0".parse().unwrap();

    // now in unix time
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let services = ServiceFlags::NONE;

    // generate random value
    let nonce = thread_rng().borrow_mut().next_u64();

    // TODO: after block database is constructed, set actual latest block height.
    let start_height = 0;

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // build message
    VersionMessage::new(
        services,
        timestamp,
        Address::new(&blank_addr, services),
        Address::new(&blank_addr, services),
        nonce,
        format!("/tapyrus-spv:{}/", VERSION),
        start_height,
    )
}
