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

mod bytes;
mod codec;


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

struct HandleConnectFuture {
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

fn handle_connect(future: ConnectFuture) -> HandleConnectFuture {
    HandleConnectFuture {
        inner: future
    }
}

fn connect(address: &str) -> HandleConnectFuture {
    let socketaddr = address.parse().unwrap();
    let connect = TcpStream::connect(&socketaddr);
    handle_connect(connect)
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

enum Error {
    IoError(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error { Error::IoError(e) }
}

pub const SERVICE_BLOCKS:u64 = 1;
pub const SERVICE_WITNESS:u64 =  1 << 3;

fn version_message() -> VersionMessage {
    let remote = "127.0.0.1:18444".parse().unwrap();
    let local = "127.0.0.1:18444".parse().unwrap();

    // now in unix time
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::prelude::{Stream, Sink};
    use tokio::net::TcpListener;
    use std::thread;
    use tokio::io::ErrorKind;
    use std::time::Duration;
    use tokio::prelude::future::FutureResult;

    fn run_dummy_remote_peer(addr: &str) -> impl Future<Item = (), Error=()>{
        let addr = addr.parse().unwrap();
        // create dummy server
        let listener = TcpListener::bind(&addr)
            .expect("unable to bind TCP listener");

        let incoming = listener.incoming();

        incoming
            .map_err(|e| eprintln!("accept failed = {:?}", e))
            .for_each(|socket| {
                trace!("incoming new connection, {:?}", socket);
                let (reader, writer) = socket.split();

                // handle reader
                let buf = vec!(0u8; 8);
                let handle_read = tokio::io::read_exact(reader, buf).map(|(reader, buf)| {
                    trace!("Remote: read from buffer {:?}", buf);
                }).map_err(|err| {
                    eprintln!("Remote: I/O error {:?}", err)
                });

                tokio::spawn(handle_read);

                // send version message
                let raw_msg = RawNetworkMessage {
                    magic: Network::Testnet.magic(),
                    payload: NetworkMessage::Version(version_message()),
                };

                let mut buffer = Vec::new();
                raw_msg.consensus_encode(&mut buffer).unwrap();

                let bytes_written = tokio::io::write_all(writer, buffer);
                let handle_conn = bytes_written.map(|amt| {
                    trace!("Remote: wrote {:?} bytes", amt)
                }).map_err(|err| {
                    eprintln!("I/O error {:?}", err)
                });

                tokio::spawn(handle_conn)
            })
    }

    fn create_client(addr: &str) -> impl tokio::prelude::Future<Item=(), Error=()> {
        connect(addr).and_then(|framed| {
            trace!("Local: Connect");

            let (sink, stream) = framed.split();
            let send = sink.send(NetworkMessage::Version(version_message()))
                .map_err(|e| println!("error = {:?}", e))
                .and_then(|a| {
                    Ok(())
                });

            tokio::spawn(send);

            stream.for_each(|msg| {
                trace!("Local Receve {:?}", msg);
                Ok(())
            })
        }).map_err(|e| println!("error = {:?}", e))
    }

    #[test]
    fn connect2_test() {
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