use crate::network::{Error, Peer};
use bitcoin::network::message::NetworkMessage;
use bitcoin::network::message_blockdata::GetHeadersMessage;
use bitcoin::{network::message::RawNetworkMessage, BitcoinHash, Network};
use bitcoin_hashes::sha256d;
use std::cell::RefCell;
use tokio::prelude::{Async, Future, Sink, Stream};
use bitcoin::blockdata::block::LoneBlockHeader;
use std::sync::{Arc, Mutex};
use bitcoin::blockdata::constants::genesis_block;
use crate::chain::ChainState;

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
    pub fn new(peer: Peer<T>, chain_state: Arc<Mutex<ChainState>>) -> BlockHeaderDownload<T> {
        BlockHeaderDownload {
            peer: Some(RefCell::new(peer)),
            started: false,
            headers_processor: HeadersProcessor { chain_state }
        }
    }

    fn get_locator(&self) -> Vec<sha256d::Hash> {
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
    pub chain_state: Arc<Mutex<ChainState>>
}

impl HeadersProcessor {
    pub fn process(&mut self, headers: Vec<LoneBlockHeader>) {
        for header in headers {
            let mut chain_state = self.chain_state.lock().unwrap();
            let chain_active = chain_state.borrow_mut_chain_active();
            let _ = chain_active.connect_block_header(header.header);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::{channel, get_test_headers, TwoWayChannel};

    /// Build remote peer for testing BlockHeaderDownload future.
    /// Remote peer checks and responds messages from local peer.
    fn remote_peer(stream: TwoWayChannel<RawNetworkMessage>) -> impl Future<Item=(), Error=()> {
        stream
            .into_future()
            .map(|(msg, mut here)| {
                match msg {
                    Some(RawNetworkMessage {
                             payload: NetworkMessage::GetHeaders(
                                 GetHeadersMessage {
                                     locator_hashes,
                                     stop_hash,
                                     ..
                                 }),
                             ..
                         }) => {
                        // test BlockHeaderDownload future send collect message.
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
            .map_err(|_| {})
    }

    #[test]
    fn blockheaderdownload_test() {
        simple_logger::init().unwrap();
        let (here, there) = channel::<RawNetworkMessage>();
        let peer = Peer::new(0, there, "0.0.0.0:0".parse().unwrap(), Network::Regtest);

        let chain_state = Arc::new(Mutex::new(ChainState::new()));
        let chain_state_for_block_header_download = chain_state.clone();

        let future = tokio::prelude::future::lazy(move || {
            tokio::spawn(remote_peer(here));

            let blockheaderdownload = BlockHeaderDownload::new(peer, chain_state_for_block_header_download)
                .map(move |_| {
                    // test after BlockHeaderDownload future finished
                    let chain_state = chain_state.lock().unwrap();
                    let chain_active = chain_state.borrow_chain_active();
                    assert_eq!(chain_active.height(), 10);
                })
                .map_err(|_| {});

            tokio::spawn(blockheaderdownload);

            Ok(())
        });

        tokio::runtime::current_thread::run(future);
    }
}