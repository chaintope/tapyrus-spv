// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

use crate::chain::{Chain, ChainStore};
use crate::network::{error::MaliciousPeerCause, Error, Peer};
use crate::ChainState;
use bitcoin::blockdata::block::LoneBlockHeader;
use bitcoin::network::message::NetworkMessage;
use bitcoin::network::message::RawNetworkMessage;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use tokio::prelude::{Async, Future, Sink, Stream};

/// The maximum number of block headers that can be in a single headers message.
pub const MAX_HEADERS_RESULTS: usize = 2_000;

pub struct BlockHeaderDownload<T, S>
where
    T: Sink<SinkItem = RawNetworkMessage> + Stream<Item = RawNetworkMessage>,
    S: ChainStore,
{
    peer: Option<RefCell<Peer<T>>>,
    started: bool,
    chain_state: Arc<Mutex<ChainState<S>>>,
    max_headers_results: usize,
}

impl<T, S> BlockHeaderDownload<T, S>
where
    T: Sink<SinkItem = RawNetworkMessage> + Stream<Item = RawNetworkMessage>,
    S: ChainStore,
{
    pub fn new(peer: Peer<T>, chain_state: Arc<Mutex<ChainState<S>>>) -> BlockHeaderDownload<T, S> {
        BlockHeaderDownload {
            peer: Some(RefCell::new(peer)),
            started: false,
            chain_state,
            max_headers_results: MAX_HEADERS_RESULTS,
        }
    }
}

/// Process received headers message.
/// Return flag for whether all block headers received.
fn process_headers<T, S: ChainStore>(
    peer: &mut Peer<T>,
    chain_active: &mut Chain<S>,
    headers: Vec<LoneBlockHeader>,
    max_headers_results: usize,
) -> Result<bool, Error>
where
    T: Sink<SinkItem = RawNetworkMessage> + Stream<Item = RawNetworkMessage>,
{
    if headers.len() > max_headers_results {
        return Err(Error::MaliciousPeer(
            peer.id,
            MaliciousPeerCause::SendOverMaxHeadersResults,
        ));
    }

    let all_headers_downloaded = headers.len() < max_headers_results;

    for header in headers {
        let _ = chain_active.connect_block_header(header.header);
    }

    if !all_headers_downloaded {
        peer.send_getheaders(chain_active);
    }

    Ok(all_headers_downloaded)
}

impl<T, S> Future for BlockHeaderDownload<T, S>
where
    T: Sink<SinkItem = RawNetworkMessage> + Stream<Item = RawNetworkMessage>,
    S: ChainStore,
    Error: From<T::Error>,
{
    type Item = Peer<T>;
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        let mut done = false;

        let mut chain_state = self.chain_state.lock().unwrap();
        let chain_active = chain_state.borrow_mut_chain_active();

        if let Some(ref peer) = self.peer {
            let mut peer = peer.borrow_mut();

            if !self.started {
                peer.send_getheaders(chain_active);
                self.started = true;
            }

            loop {
                match peer.poll()? {
                    Async::Ready(Some(NetworkMessage::Headers(headers))) => {
                        done = process_headers(
                            &mut peer,
                            chain_active,
                            headers,
                            self.max_headers_results,
                        )?;
                    }
                    Async::Ready(None) | Async::NotReady => break,
                    Async::Ready(_) => {} // ignore other messages.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::{
        channel, get_chain, get_test_headers, get_test_lone_headers, TwoWayChannel,
    };
    use bitcoin::blockdata::constants::genesis_block;
    use bitcoin::network::message_blockdata::GetHeadersMessage;
    use bitcoin::{BitcoinHash, Network};
    use bitcoin_hashes::sha256d;

    #[test]
    fn test_process_headers_fails_when_passed_over_max_headers_results() {
        let (_here, there) = channel::<RawNetworkMessage>();
        let mut peer = Peer::new(0, there, "0.0.0.0:0".parse().unwrap(), Network::Regtest);

        let mut chain_state = ChainState::new(get_chain());
        let mut chain_active = chain_state.borrow_mut_chain_active();
        let headers = get_test_lone_headers(1, 11);
        let result = process_headers(&mut peer, &mut chain_active, headers, 10);

        assert!(result.is_err());
        match result {
            Err(Error::MaliciousPeer(peer_id, _)) => assert_eq!(peer_id, 0),
            _ => assert!(false),
        }
    }

    /// Build remote peer for testing BlockHeaderDownload future.
    /// Remote peer checks and responds messages from local peer.
    ///
    /// ## Situation
    /// Local peer has only genesis block. Remote peer has 24 blocks includes genesis block.
    ///
    /// ## Flow
    /// 1st message round trip, local peer send getheaders message and get 10 blocks from remote.
    /// 2nd message round trip, local peer send getheaders message and get 10 blocks from remote.
    /// 3rd message round trip, local peer send getheaders message and get 3 blocks from remote.
    /// And finish sending getheaders message.
    fn remote_peer(stream: TwoWayChannel<RawNetworkMessage>) -> impl Future<Item = (), Error = ()> {
        stream
            .into_future()
            .and_then(|(msg, mut here)| {
                // 1st message round trip.
                // local peer request block headers with `getheaders` message. And remote peer sends
                // MAX_HEADERS_RESULTS headers.
                if let Some(RawNetworkMessage {
                    payload: NetworkMessage::GetHeaders(getheaders_msg),
                    ..
                }) = msg
                {
                    match getheaders_msg {
                        GetHeadersMessage {
                            locator_hashes,
                            stop_hash,
                            ..
                        } => {
                            // test BlockHeaderDownload future send collect message.
                            assert_eq!(
                                locator_hashes,
                                vec![genesis_block(Network::Regtest).header.bitcoin_hash()]
                            );
                            assert_eq!(stop_hash, sha256d::Hash::default());
                        }
                    }
                } else {
                    assert!(false, "Peer should send 1st getheaders message.");
                }

                let headers_message = RawNetworkMessage {
                    magic: Network::Regtest.magic(),
                    payload: NetworkMessage::Headers(get_test_lone_headers(1, 10)),
                };

                let _ = here.start_send(headers_message);

                here.into_future()
            })
            .and_then(|(msg, mut here)| {
                // 2nd message round trip.
                // Remote peer send next 10 headers.
                if let Some(RawNetworkMessage {
                    payload: NetworkMessage::GetHeaders(getheaders_msg),
                    ..
                }) = msg
                {
                    match getheaders_msg {
                        GetHeadersMessage {
                            locator_hashes,
                            stop_hash,
                            ..
                        } => {
                            // test BlockHeaderDownload future send collect message.
                            let expected: Vec<sha256d::Hash> = get_test_headers(0, 11)
                                .into_iter()
                                .rev()
                                .map(|v| v.bitcoin_hash())
                                .collect();
                            assert_eq!(locator_hashes, expected);
                            assert_eq!(stop_hash, sha256d::Hash::default());
                        }
                    }
                } else {
                    assert!(false, "Peer should send 2nd getheaders message.");
                }

                let headers_message = RawNetworkMessage {
                    magic: Network::Regtest.magic(),
                    payload: NetworkMessage::Headers(get_test_lone_headers(10, 10)),
                };

                let _ = here.start_send(headers_message);

                here.into_future()
            })
            .map(|(msg, mut here)| {
                // 3rd message round trip.
                // Remote peer send 3 headers as latest headers.
                if let Some(RawNetworkMessage {
                    payload: NetworkMessage::GetHeaders(getheaders_msg),
                    ..
                }) = msg
                {
                    match getheaders_msg {
                        GetHeadersMessage { stop_hash, .. } => {
                            // test BlockHeaderDownload future send collect message.
                            assert_eq!(stop_hash, sha256d::Hash::default());
                        }
                    }
                } else {
                    assert!(false, "Peer should send 3rd getheaders message.");
                }

                let headers_message = RawNetworkMessage {
                    magic: Network::Regtest.magic(),
                    payload: NetworkMessage::Headers(get_test_lone_headers(20, 3)),
                };

                let _ = here.start_send(headers_message);

                ()
            })
            .map_err(|_| {})
    }

    #[test]
    fn test_block_header_download() {
        let (here, there) = channel::<RawNetworkMessage>();
        let peer = Peer::new(0, there, "0.0.0.0:0".parse().unwrap(), Network::Regtest);

        let chain_state = Arc::new(Mutex::new(ChainState::new(get_chain())));
        let chain_state_for_block_header_download = chain_state.clone();

        let future = tokio::prelude::future::lazy(move || {
            tokio::spawn(remote_peer(here));

            let blockheaderdownload = BlockHeaderDownload {
                peer: Some(RefCell::new(peer)),
                started: false,
                chain_state: chain_state_for_block_header_download,
                max_headers_results: 10,
            }
            .map(move |_| {
                // test after BlockHeaderDownload future finished
                let chain_state = chain_state.lock().unwrap();
                let chain_active = chain_state.borrow_chain_active();
                assert_eq!(chain_active.height(), 23);
            })
            .map_err(|_| {});

            tokio::spawn(blockheaderdownload);

            Ok(())
        });

        tokio::runtime::current_thread::run(future);
    }
}
