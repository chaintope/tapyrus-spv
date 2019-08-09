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
use tokio::prelude::future::result;


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
    type Item = (ReadHalf<TcpStream>, WriteHalf<TcpStream>);
    type Error = io::Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, io::Error> {
        match self.inner.poll()? {
            Async::Ready(stream) => Ok(Async::Ready(stream.split())),
            Async::NotReady => Ok(Async::NotReady),
        }
    }
}

fn handle_connect(future: ConnectFuture) -> HandleConnectFuture {
    HandleConnectFuture {
        inner: future
    }
}

fn connect2(address: &str) -> HandleConnectFuture {
    let socketaddr = address.parse().unwrap();
    let connect = TcpStream::connect(&socketaddr);
    handle_connect(connect)
}

fn connect<'a>(address: &'a str) -> impl Future<Item = Peer, Error = std::io::Error> + 'a {
    trace!("Local: connect");
    let socketaddr = address.parse().unwrap();
    TcpStream::connect(&socketaddr)
        .and_then(move |stream| {
            let (mut reader, writer) = stream.split();

            trace!("Local: connected");
            let future = tokio::prelude::future::lazy(move || -> tokio::prelude::future::FutureResult<(), ()> {
                loop {
                    trace!("Local: reader loop");
                    match StreamReader::new(&mut reader, None).next_message() {
                        Ok(msg) => {
                            trace!("Local: get messag from peer: {:?}", msg);
                        }
                        a => {
                            trace!("read: {:?}", a);
                        }
                    }

                }
            });



            // create peer
            let socketaddr = address.parse().unwrap();
            let peer = Peer::new(1, &socketaddr, writer);
            result(Ok(peer))
        })
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

//    fn connect<'a>(&'a mut self) -> impl Future<Item = tokio::net::TcpStream, Error = std::io::Error> + 'a {
//        TcpStream::connect(&self.address)
//            .and_then(|stream| {
//                self.socket = Some(stream);
//                stream
//            })
//    }

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
    use tokio::prelude::Stream;
    use tokio::net::TcpListener;
    use std::thread;
    use tokio::io::ErrorKind;
    use std::time::Duration;

    #[test]
    fn connect2_test() {
        simple_logger::init().unwrap();

        let con = connect2("127.0.0.1:18444");

        let result = con.wait();
        println!("{:?}", result);
    }

    #[test]
    fn connect_test() {
        simple_logger::init().unwrap();

        let con = connect("127.0.0.1:18444");

        let _ = con.wait();
    }

    fn run_dummy_remote_peer(addr: &SocketAddr) {
        // create dummy server
        let listener = TcpListener::bind(addr)
            .expect("unable to bind TCP listener");

        let incoming = listener.incoming();

        let server = incoming
            .map_err(|e| eprintln!("accept failed = {:?}", e))
            .for_each(|socket| {
                trace!("incoming new connection, {:?}", socket);
                let (reader, writer) = socket.split();


                // handle reader
                let buf = vec!(0u8; 1024 * 1024);
                let handle_read = tokio::io::read_exact(reader, buf).map(|(reader, buf)| {
                    trace!("Remote: read from buffer {:?}", buf);
                }).map_err(|err| {
                    eprintln!("I/O error {:?}", err)
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
            });

        tokio::run(server);
    }

    #[test]
    fn handle_message_test() {
        simple_logger::init().unwrap();

        let remote = thread::spawn(move || {
            run_dummy_remote_peer(&("127.0.0.1:18555".parse().unwrap()))
        });
        trace!("Dummy Server Started");

//        let peer;

        use tokio::{
            runtime::Runtime,
            prelude::future::{
                FutureResult,
                lazy,
                result,
            },
        };

        let mut runtime = Runtime::new().unwrap();

        let result = runtime.block_on(lazy(|| -> FutureResult<Peer, std::io::Error> {
            trace!("Local: waiting connection");
            let mut connect_future = connect("127.0.0.1:18555");



            loop {
                match connect_future.poll() {
                    Ok(Async::Ready(peer)) => {
                        trace!("Local: peer: {:?}", peer);
                        return result(Ok(peer))
                    }
                    Ok(Async::NotReady) => {
                        trace!("Local: Not Ready");
                    },
                    Err(e) => {
                        trace!("Local: Not Ok, {}", e);
                    }
                }

                thread::sleep(Duration::new(1, 0));
            }
        }));

//        if let FutureResult({inner: Some(p)}) = result {
//
//        }

//        futures::executor::block_on(|&mut context| {
//            match connect_future.poll(context) {
//                Ok(Async::Ready(peer)) => {
//                    trace!("{:?}", peer);
//                },
//                _ => {
//                    trace!("Not Ok");
//                }
//            }
//        });
//        futures::(futures::future::lazy(|&mut context| {
//            match connect_future.compat().poll(context) {
//                Ok(Async::Ready(peer)) => {
//                    trace!("{:?}", peer);
//                },
//                _ => {
//                    trace!("Not Ok");
//                }
//            }
//        })).wait_future();

//        connect_future.
//
//        let mut stream = connect_future.into_stream();
//        loop {
//            match stream.poll() {
//                Ok(Async::Ready(Some(p))) => {
//                    trace!("{:?}", p);
//                },
//                _ => {}
//            }
//        }

//        let mut peer = Peer::new(1, &addr);
//        let result  = peer.connect().wait();

//        let hoge = StreamReader::new(&mut read, None).next_message();
//        trace!("{:?}", hoge);

        trace!("Connected");


        let _ = remote.join();
    }

}