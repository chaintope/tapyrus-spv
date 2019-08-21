use tokio::{prelude::*, net::{
    TcpStream,
    tcp::ConnectFuture,
}, codec::Framed};
use crate::network::codec::NetworkMessagesCodec;
use crate::network::{codec, Peer};

pub struct HandleConnectFuture {
    inner: ConnectFuture
}

pub type NetworkMessageStream = Framed<TcpStream, NetworkMessagesCodec>;

impl Future for HandleConnectFuture
{
    type Item = Peer;
    type Error = codec::Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, codec::Error> {
        match self.inner.poll()? {
            Async::Ready(stream) => {
                let peer = Peer::new(0, stream);

                Ok(Async::Ready(peer))
            },
            Async::NotReady => Ok(Async::NotReady),
        }
    }
}

pub fn connect(address: &str) -> HandleConnectFuture {
    let socketaddr = address.parse().unwrap();
    let connect = TcpStream::connect(&socketaddr);

    HandleConnectFuture {
        inner: connect
    }
}