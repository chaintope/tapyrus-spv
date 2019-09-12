use crate::network::{Error, Peer};
use bitcoin::network::message::NetworkMessage;
use bitcoin::network::message_blockdata::GetHeadersMessage;
use bitcoin::{network::message::RawNetworkMessage, BitcoinHash, BlockHeader, Network};
use bitcoin_hashes::sha256d;
use std::cell::RefCell;
use tokio::prelude::{Async, Future, Sink, Stream};
use bitcoin::blockdata::block::LoneBlockHeader;
use std::sync::{Arc, Mutex};
use std::borrow::Borrow;
use bitcoin::blockdata::constants::genesis_block;
use bitcoin::consensus::serialize;

pub struct BlockHeaderDownload<T>
where
    T: Sink<SinkItem = RawNetworkMessage> + Stream<Item = RawNetworkMessage>,
{
    peer: Option<RefCell<Peer<T>>>,
    started: bool,
    headers_processor: HeadersProcessor
}

impl<T> BlockHeaderDownload<T>
where
    T: Sink<SinkItem = RawNetworkMessage> + Stream<Item = RawNetworkMessage>,
{
    pub fn new(peer: Peer<T>, headers: Arc<Mutex<Vec<BlockHeader>>>) -> BlockHeaderDownload<T> {
        BlockHeaderDownload {
            peer: Some(RefCell::new(peer)),
            started: false,
            headers_processor: HeadersProcessor { headers }
        }
    }

    fn get_locator(&self) -> Vec<sha256d::Hash> {
//        let headers = self.headers_processor.headers();
        //vec![headers[0].bitcoin_hash()]
        let genesis = genesis_block(Network::Regtest);
        vec![genesis.header.bitcoin_hash()]
    }

    fn send_getheaders(&self, peer: &mut Peer<T>) {
        let locators = self.get_locator();
        let stop_hash = sha256d::Hash::default();
        let getheaders = GetHeadersMessage::new(locators, stop_hash);
        peer.start_send(NetworkMessage::GetHeaders(getheaders));
    }
}

impl<T> Future for BlockHeaderDownload<T>
where
    T: Sink<SinkItem = RawNetworkMessage> + Stream<Item = RawNetworkMessage>,
    Error: From<T::Error>,
{
    type Item = Peer<T>;
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        let mut done = false;

        if let Some(ref peer) = self.peer {
            let mut peer = peer.borrow_mut();

            if !self.started {
                self.send_getheaders(&mut peer);
                self.started = true;
            }

            loop {
                match peer.poll()? {
                    Async::Ready(Some(NetworkMessage::Headers(headers))) => {
                        self.headers_processor.process(headers);
                        done = true;
                    }
                    Async::Ready(None) => break,
                    Async::Ready(_) => {}, // ignore other messages.
                    Async::NotReady => break,
                }
            }
            peer.flush();
        } else {
            panic!("BlockHeaderDownload should have peer instance when call poll.");
        }

        if done {
            let peer = self.peer.take().unwrap();
            Ok(Async::Ready(peer.into_inner()))
        } else {
            Ok(Async::NotReady)
        }
    }
}

struct HeadersProcessor {
    pub headers: Arc<Mutex<Vec<BlockHeader>>>
}

use hex::encode as hex_encode;

impl HeadersProcessor {
    pub fn process(&mut self, headers: Vec<LoneBlockHeader>) {
        for header in headers {
            let mut headers = self.headers.lock().unwrap();
            headers.push(header.header);
        }
    }

//    pub fn headers(&self) -> &Vec<BlockHeader> {
//        self.headers.lock().unwrap()
//    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::{channel, get_test_headers};

    #[test]
    fn blockheaderdownload_test() {
        simple_logger::init().unwrap();
        let (here, there) = channel::<RawNetworkMessage>();
        let peer = Peer::new(0, there, "0.0.0.0:0".parse().unwrap(), Network::Regtest);
        let genesis = genesis_block(Network::Regtest);
        let headers = Arc::new(Mutex::new(vec![genesis.header]));

        let future = tokio::prelude::future::lazy(move || {
            let blockheaderdownload = BlockHeaderDownload::new(peer, headers)
                .map(|_| {})
                .map_err(|_| {});
            tokio::spawn(blockheaderdownload);

            let test_future = here
                .into_future()
                .map(|(msg, mut here)| {
                    match msg {
                        Some(RawNetworkMessage {
                            payload: NetworkMessage::GetHeaders(
                                GetHeadersMessage {
                                    locator_hashes: locator_hashes,
                                    stop_hash: stop_hash,
                                    ..
                                }),
                            ..
                        }) => {
                            assert_eq!(locator_hashes, vec![genesis_block(Network::Regtest).header.bitcoin_hash()]);
                            assert_eq!(stop_hash, sha256d::Hash::default());
                        },
                        _ => assert!(false)
                    }

                    let headers_message = RawNetworkMessage {
                        magic: Network::Regtest.magic(),
                        payload: NetworkMessage::Headers(get_test_headers(0, 10))
                    };

                    let _ = here.start_send(headers_message);
                    ()
                })
                .map_err(|_| {});

            tokio::spawn(test_future);

            Ok(())
        });

        tokio::runtime::current_thread::run(future);
    }
}