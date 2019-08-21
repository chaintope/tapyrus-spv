use tokio::{prelude::*, net::{
    TcpStream,
    tcp::ConnectFuture,
}, codec::Framed};
use bitcoin::network::constants::Network;
use crate::network::codec::NetworkMessagesCodec;
use crate::network::{codec, Peer};

pub struct HandleConnectFuture {
    inner: ConnectFuture,
    network: Network,
}

pub type NetworkMessageStream = Framed<TcpStream, NetworkMessagesCodec>;

impl Future for HandleConnectFuture
{
    type Item = Peer;
    type Error = codec::Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, codec::Error> {
        match self.inner.poll()? {
            Async::Ready(stream) => {
                let peer = Peer::new(0, stream, self.network);

                Ok(Async::Ready(peer))
            },
            Async::NotReady => Ok(Async::NotReady),
        }
    }
}

pub fn connect(address: &str, network: Network) -> HandleConnectFuture {
    let socketaddr = address.parse().unwrap();
    let connect = TcpStream::connect(&socketaddr);

    HandleConnectFuture {
        inner: connect,
        network,
    }
}