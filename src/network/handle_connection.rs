use tokio::{
    prelude::*,
    net::{
        TcpStream,
        tcp::ConnectFuture,
    },
    codec::Framed,
};
use crate::network::codec::NetworkMessagesCodec;
use crate::network::codec;

pub struct HandleConnectFuture {
    inner: ConnectFuture
}

impl Future for HandleConnectFuture
{
    type Item = Framed<TcpStream, NetworkMessagesCodec>;
    type Error = codec::Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, codec::Error> {
        match self.inner.poll()? {
            Async::Ready(stream) => {
                let framed_sock = Framed::new(stream, NetworkMessagesCodec::new());
                Ok(Async::Ready(framed_sock))
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