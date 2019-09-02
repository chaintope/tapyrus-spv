use crate::network::{codec::NetworkMessagesCodec, Error};
use bitcoin::network::{
    address::Address,
    constants::Network,
    message::{NetworkMessage, RawNetworkMessage},
    message_network::VersionMessage,
};
use rand::{thread_rng, RngCore};
use std::{
    borrow::BorrowMut,
    net::SocketAddr,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{codec::Framed, net::TcpStream, prelude::*};

pub struct Peer<T>
where
    T: Sink<SinkItem = RawNetworkMessage> + Stream<Item = RawNetworkMessage>,
{
    pub id: u64,
    pub addr: SocketAddr,
    pub network: Network,
    pub stream: T,
    pub version: Option<VersionMessage>,
}

impl<T> Peer<T>
where
    T: Sink<SinkItem = RawNetworkMessage> + Stream<Item = RawNetworkMessage>,
{
    pub fn new(id: u64, stream: T, addr: SocketAddr, network: Network) -> Peer<T> {
        Peer {
            id,
            addr,
            network,
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
            magic: self.network.magic(),
            payload: message,
        };

        let _ = self.stream.start_send(raw_msg);
    }

    /// flush all queued sending messages.
    pub fn flush(&mut self) {
        let _ = self.stream.poll_complete();
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
                if message.magic != self.network.magic() {
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
    address: &str,
    network: Network,
) -> impl Future<Item = Peer<Framed<TcpStream, NetworkMessagesCodec>>, Error = Error> {
    let socketaddr = address.parse().unwrap();

    TcpStream::connect(&socketaddr)
        .map(move |stream| {
            let addr = stream.peer_addr().unwrap();
            let stream = Framed::new(stream, NetworkMessagesCodec::new());
            Peer::new(0, stream, addr, network)
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

    let services = 0;

    // generate random value
    let nonce = thread_rng().borrow_mut().next_u64();

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
        start_height,
    )
}
