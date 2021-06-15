// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

use crate::network::peer::version_message;
use crate::network::{Error, Peer};
use tapyrus::network::message::{NetworkMessage, RawNetworkMessage};
use tokio::prelude::*;

pub struct Handshake<T>
where
    T: Sink<SinkItem = RawNetworkMessage> + Stream<Item = RawNetworkMessage>,
{
    peer: Option<Peer<T>>,
    sent_version: bool,
    received_version: bool,
    received_verack: bool,
}

impl<T> Handshake<T>
where
    T: Sink<SinkItem = RawNetworkMessage> + Stream<Item = RawNetworkMessage>,
{
    pub fn new(peer: Peer<T>) -> Handshake<T> {
        Handshake {
            peer: Some(peer),
            sent_version: false,
            received_version: false,
            received_verack: false,
        }
    }
}

impl<T> Future for Handshake<T>
where
    T: Sink<SinkItem = RawNetworkMessage> + Stream<Item = RawNetworkMessage>,
    Error: From<T::Error>,
{
    type Item = Peer<T>;
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        if let Some(ref mut peer) = self.peer {
            if !self.sent_version {
                peer.start_send(NetworkMessage::Version(version_message()));
                self.sent_version = true;
            }

            loop {
                match peer.poll()? {
                    Async::Ready(Some(NetworkMessage::Version(version))) => {
                        peer.version = Some(version);

                        // send verack message
                        let _ = peer.start_send(NetworkMessage::Verack);
                        self.received_version = true;
                    }
                    Async::Ready(Some(NetworkMessage::Verack)) => {
                        self.received_verack = true;
                    }
                    Async::Ready(None) => break,
                    Async::Ready(_) => {} // ignore other messages.
                    Async::NotReady => break,
                }
            }

            peer.flush();
        } else {
            panic!("Handshake should have peer instance when call poll.");
        }

        // check either handshake finished
        if self.sent_version && self.received_version && self.received_verack {
            let peer = self.peer.take().unwrap();
            trace!("Handshake complete. peer: {}", peer.id);
            Ok(Async::Ready(peer))
        } else {
            Ok(Async::NotReady)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::channel;
    use tapyrus::network::constants::Network;

    #[test]
    fn test_handshake() {
        let (here, there) = channel::<RawNetworkMessage>();

        let addr = "0.0.0.0:0".parse().unwrap();
        let peer = Peer::new(0, there, addr, Network::Dev);

        let future = tokio::prelude::future::lazy(move || {
            let handshake = Handshake::new(peer).map(|_| {}).map_err(|_| {});

            tokio::spawn(handshake);

            let test_future = here
                .into_future()
                .and_then(|(msg, mut here)| {
                    // check version message received.
                    match msg {
                        Some(RawNetworkMessage {
                            payload: NetworkMessage::Version(_),
                            ..
                        }) => {
                            assert!(true);
                        }
                        _ => assert!(false),
                    }

                    // send version message.
                    let version = RawNetworkMessage {
                        magic: Network::Dev.magic(),
                        payload: NetworkMessage::Version(version_message()),
                    };

                    let _ = here.start_send(version);

                    // send verack message.
                    let verack = RawNetworkMessage {
                        magic: Network::Dev.magic(),
                        payload: NetworkMessage::Verack,
                    };
                    let _ = here.start_send(verack);

                    // flush sending message.
                    let _ = here.poll_complete();

                    here.into_future()
                })
                .map(|(msg, _here)| {
                    // check verack message received.
                    match msg {
                        Some(RawNetworkMessage {
                            payload: NetworkMessage::Verack,
                            ..
                        }) => {
                            assert!(true);
                        }
                        _ => assert!(false),
                    }
                })
                .map_err(|_| {});

            tokio::spawn(test_future);

            Ok(())
        });

        tokio::runtime::current_thread::run(future);
    }
}
