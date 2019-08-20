use tokio::{prelude::*, net::{
    TcpStream,
    tcp::ConnectFuture,
}, codec::Framed};
use crate::network::codec::NetworkMessagesCodec;
use crate::network::{codec, Peer, version_message};
use bitcoin::network::message::NetworkMessage::Version;
use tokio::prelude::future::Either;
use std::io;
use bitcoin::network::message::NetworkMessage;

pub struct HandleConnectFuture {
    inner: ConnectFuture
}

pub type NetworkMessageStream = Framed<TcpStream, NetworkMessagesCodec>;
//pub type NetworkMessageStream = Box<dyn Stream<Item = NetworkMessage, Error = codec::Error> + Send>;
//pub type NetworkMessageSink = Box<dyn Sink<SinkItem = NetworkMessage, SinkError = io::Error> + Send>;

impl Future for HandleConnectFuture
{
    type Item = Peer;
    type Error = codec::Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, codec::Error> {
        match self.inner.poll()? {
            Async::Ready(stream) => {
                let addr = stream.peer_addr().unwrap();
//                let framed = Framed::new(stream, NetworkMessagesCodec::new());
//                let (sink, stream) = framed.split();
                let peer = Peer::new(0, stream);

//                let send = sink.send(NetworkMessage::Version(version_message()))
//                    .and_then(|sink| {
//                        trace!("Send version message. {:?}", sink);
//                        Ok(())
//                    })
//                    .map_err(|e| {
//                        error!("Error: {}", e);
//                    });
//                tokio::spawn(send);
//
//                let reading = stream
//                    .for_each(|message| {
//                        match message {
//                            NetworkMessage::Version(version) => {
//                                trace!("Receive version message: {:?}", version);
//                            }
//                            m => { trace!("Receive message: {:?}", m) }
//                        }
//                        Ok(())
//                    })
//                    .map_err(|e| println!("error = {:?}", e));
//                tokio::spawn(reading);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[test]
    fn connect_test() {
        simple_logger::init().unwrap();

        let future = tokio::prelude::future::lazy(|| {
            let remote = run_dummy_remote_peer("127.0.0.1:18555");
            tokio::spawn(remote);
            let client = create_client("127.0.0.1:18555");
            tokio::spawn(client)
        });

        tokio::run(future);
    }
}