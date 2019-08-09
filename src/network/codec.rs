use tokio::codec::{Encoder, Decoder};
use bitcoin::{
    consensus::{Encodable, deserialize_partial, encode},
    network::{
        message::{NetworkMessage, RawNetworkMessage},
        constants::Network,
    }
};
use std::{
    io,
    io::ErrorKind,
    sync::atomic::AtomicUsize,
};
use super::bytes::BytesMut;

#[derive(Debug)]
pub enum Error {
    Encode(encode::Error),
    Io(io::Error),
}

impl std::convert::From<std::io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

/// Codec for bytes stream carrying NetworkMessage.
pub struct NetworkMessagesCodec {}

impl NetworkMessagesCodec {
    pub fn new() -> NetworkMessagesCodec { NetworkMessagesCodec{} }
}

impl Decoder for NetworkMessagesCodec {
    type Item = NetworkMessage;
    type Error = Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<NetworkMessage>, Error> {
        match deserialize_partial::<RawNetworkMessage>(& src) {
            Ok((raw_msg, consumed)) => {
                src.advance(consumed);
                Ok(Some(raw_msg.payload))
            }
            Err(encode::Error::Io(ref e)) if e.kind() == ErrorKind::UnexpectedEof => Ok(None),
            Err(e) => Err(Error::Encode(e)),
        }
    }
}

impl Encoder for NetworkMessagesCodec {
    type Item = NetworkMessage;
    type Error = io::Error;

    fn encode(&mut self, message: NetworkMessage, buf: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        let raw_msg = RawNetworkMessage {
            magic: Network::Testnet.magic(),
            payload: message,
        };

        let mut buf = BytesMut::new(buf);

        raw_msg.consensus_encode(&mut buf).unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::network::{
        message_network::VersionMessage,
        address::Address,
    };
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::sync::atomic::Ordering;
    use crate::network::{SERVICE_BLOCKS, SERVICE_WITNESS};
    use rand::{thread_rng, RngCore};
    use bytes::BufMut;
    use bitcoin::network::message::NetworkMessage::Version;


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

    #[test]
    fn decode_test() {
        let data = [
            0x0b, 0x11, 0x09, 0x07, 0x76, 0x65, 0x72, 0x73,
            0x69, 0x6f, 0x6e, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x69, 0x00, 0x00, 0x00, 0x3e, 0x1d, 0xe1, 0x69,
            0x71, 0x11, 0x01, 0x00, 0x09, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x19, 0x3b, 0x4d, 0x5d,
            0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff,
            0x7f, 0x00, 0x00, 0x01, 0x48, 0x0c, 0x01, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0xff, 0xff, 0x7f, 0x00, 0x00, 0x01, 0x48, 0x0c,
            0xf3, 0xa0, 0x62, 0xa8, 0xb3, 0x2b, 0x38, 0x92,
            0x13, 0x2f, 0x62, 0x69, 0x74, 0x63, 0x6f, 0x69,
            0x6e, 0x2d, 0x73, 0x70, 0x76, 0x3a, 0x30, 0x2e,
            0x31, 0x2e, 0x30, 0x2f, 0x00, 0x00, 0x00, 0x00,
            0x00,
            // dummy bytes
            0x0b, 0x11, 0x09, 0x07, 0x76, 0x65, 0x72, 0x73,
        ];

        let mut codec = NetworkMessagesCodec::new();
        let mut buf = bytes::BytesMut::with_capacity(1024);
        buf.put_slice(&data);

        if let Ok(Some(Version(msg))) = codec.decode(&mut buf) {
            assert_eq!(msg.user_agent, "/bitcoin-spv:0.1.0/".to_string());
        } else {
            assert!(false);
        }

        assert_eq!(buf.len(), 8);
        assert_eq!(buf.to_vec(), vec![0x0b, 0x11, 0x09, 0x07, 0x76, 0x65, 0x72, 0x73]);
    }

    #[test]
    fn decode_incompleteness_data_test() {
        let data = [
            0x0b, 0x11, 0x09, 0x07, 0x76, 0x65, 0x72, 0x73,
            0x69, 0x6f, 0x6e, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x69, 0x00, 0x00, 0x00, 0x3e, 0x1d, 0xe1, 0x69,
        ];

        let mut codec = NetworkMessagesCodec::new();
        let mut buf = bytes::BytesMut::with_capacity(1024);
        buf.put_slice(&data);

        // if the bytes in buffer need more data, returns OK(None).
        assert!(codec.decode(&mut buf).unwrap().is_none());
    }

    #[test]
    fn encode_test() {
        let msg = NetworkMessage::Version(version_message());
        let mut codec = NetworkMessagesCodec::new();

        let mut buf = bytes::BytesMut::with_capacity(1024);

        assert!(codec.encode(msg, &mut buf).is_ok());
    }
}
