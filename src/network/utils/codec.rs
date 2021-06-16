// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

use super::bytes::BytesMut;
use byteorder::{LittleEndian, ReadBytesExt};
use std::{io, io::ErrorKind};
use tapyrus::{
    consensus::{deserialize_partial, encode, Encodable},
    network::message::RawNetworkMessage,
};
use tokio::codec::{Decoder, Encoder};

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
#[derive(Debug)]
pub struct NetworkMessagesCodec {}

impl NetworkMessagesCodec {
    pub fn new() -> NetworkMessagesCodec {
        NetworkMessagesCodec {}
    }
}

impl Decoder for NetworkMessagesCodec {
    type Item = RawNetworkMessage;
    type Error = Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<RawNetworkMessage>, Error> {
        match deserialize_partial::<RawNetworkMessage>(&src) {
            Ok((raw_msg, consumed)) => {
                src.advance(consumed);
                Ok(Some(raw_msg))
            }
            Err(encode::Error::Io(ref e)) if e.kind() == ErrorKind::UnexpectedEof => Ok(None),
            Err(encode::Error::UnrecognizedNetworkCommand(cmd)) => {
                warn!("Received unrecognized network command {}", cmd);

                // Skip unrecognized message.
                // rust-tapyrus cargo still has unsupporting messages which defined in network
                // protocol like `cmpctblock`, `feefilter`. So it is skipped so far.
                src.advance(4 + 12); // magic(4bytes) + command string(12bytes)
                let payload_size = {
                    let mut decoder = io::Cursor::new(&src);
                    ReadBytesExt::read_u32::<LittleEndian>(&mut decoder)? as usize
                };
                src.advance(4 + 4 + payload_size); // length(4bytes) + checksum(4bytes) + payload

                Ok(None)
            }
            Err(e) => Err(Error::Encode(e)),
        }
    }
}

impl Encoder for NetworkMessagesCodec {
    type Item = RawNetworkMessage;
    type Error = io::Error;

    fn encode(
        &mut self,
        message: RawNetworkMessage,
        buf: &mut bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        let mut buf = BytesMut::new(buf);

        message.consensus_encode(&mut buf).unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::peer::version_message;
    use bytes::BufMut;
    use tapyrus::network::constants::NetworkId;
    use tapyrus::network::message::NetworkMessage;

    #[test]
    fn decode_test() {
        let data: [u8; 137] = [
            0x0b, 0x11, 0x09, 0x07, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x69, 0x00, 0x00, 0x00, 0x3e, 0x1d, 0xe1, 0x69, 0x71, 0x11, 0x01, 0x00,
            0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0x3b, 0x4d, 0x5d, 0x00, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0x7f, 0x00, 0x00, 0x01, 0x48, 0x0c,
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0x7f, 0x00, 0x00, 0x01, 0x48, 0x0c, 0xf3, 0xa0,
            0x62, 0xa8, 0xb3, 0x2b, 0x38, 0x92, 0x13, 0x2f, 0x62, 0x69, 0x74, 0x63, 0x6f, 0x69,
            0x6e, 0x2d, 0x73, 0x70, 0x76, 0x3a, 0x30, 0x2e, 0x31, 0x2e, 0x30, 0x2f, 0x00, 0x00,
            0x00, 0x00, 0x00, // dummy bytes
            0x0b, 0x11, 0x09, 0x07, 0x76, 0x65, 0x72, 0x73,
        ];

        let mut codec = NetworkMessagesCodec::new();
        let mut buf = bytes::BytesMut::with_capacity(1024);
        buf.put_slice(&data);

        if let Ok(Some(RawNetworkMessage {
            payload: NetworkMessage::Version(msg),
            ..
        })) = codec.decode(&mut buf)
        {
            assert_eq!(msg.user_agent, "/bitcoin-spv:0.1.0/".to_string());
        } else {
            assert!(false);
        }

        assert_eq!(buf.len(), 8);
        assert_eq!(
            buf.to_vec(),
            vec![0x0b, 0x11, 0x09, 0x07, 0x76, 0x65, 0x72, 0x73]
        );
    }

    #[test]
    fn decode_unrecognized_command() {
        let data: [u8; 33] = [
            0x73, 0x9a, 0x97, 0x74, 0x73, 0x65, 0x6e, 0x64, 0x63, 0x6d, 0x70, 0x63, 0x74, 0x00,
            0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0xcc, 0xfe, 0x10, 0x4a, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let mut codec = NetworkMessagesCodec::new();
        let mut buf = bytes::BytesMut::with_capacity(1024);
        buf.put_slice(&data);

        if let Ok(None) = codec.decode(&mut buf) {
            assert_eq!(buf.len(), 0);
        } else {
            assert!(false, "decode should return `Ok(None)`");
        }
    }

    #[test]
    fn decode_incompleteness_data_test() {
        let data: [u8; 24] = [
            0x0b, 0x11, 0x09, 0x07, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x69, 0x00, 0x00, 0x00, 0x3e, 0x1d, 0xe1, 0x69,
        ];

        let mut codec = NetworkMessagesCodec::new();
        let mut buf = bytes::BytesMut::with_capacity(1024);
        buf.put_slice(&data);

        // if the bytes in buffer need more data, returns OK(None).
        assert!(codec.decode(&mut buf).unwrap().is_none());
    }

    #[test]
    fn encode_test() {
        let msg = RawNetworkMessage {
            magic: NetworkId::REGTEST.magic(),
            payload: NetworkMessage::Version(version_message()),
        };

        let mut codec = NetworkMessagesCodec::new();

        let mut buf = bytes::BytesMut::with_capacity(1024);

        assert!(codec.encode(msg, &mut buf).is_ok());
    }
}
