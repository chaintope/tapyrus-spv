// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

use tapyrus::consensus::{Decodable, Encodable};
use tapyrus::BlockHeader;
use bitcoin_hashes::sha256d;

/// This struct is a index of block header. It has not only block header but also meta data like
/// 'height', 'next_blockhash' for that.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockIndex {
    pub header: BlockHeader,
    pub height: i32,
    pub next_blockhash: sha256d::Hash,
}

impl<S: Encoder> Encodable<S> for BlockIndex {
    #[inline]
    fn consensus_encode(&self, s: &mut S) -> Result<(), bitcoin::consensus::encode::Error> {
        self.header.consensus_encode(s)?;
        self.height.consensus_encode(s)?;
        self.next_blockhash.consensus_encode(s)?;
        Ok(())
    }
}

impl<D: Decoder> Decodable<D> for BlockIndex {
    #[inline]
    fn consensus_decode(d: &mut D) -> Result<BlockIndex, bitcoin::consensus::encode::Error> {
        Ok(BlockIndex {
            header: Decodable::consensus_decode(d)?,
            height: Decodable::consensus_decode(d)?,
            next_blockhash: Decodable::consensus_decode(d)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::get_test_block_index;
    use tapyrus::consensus::{deserialize, serialize};

    const SERIALIZED_GENESIS_BLOCK_INDEX: &str = "0100000000000000000000000000000000000000000000000000000000000000000000003ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4adae5494dffff7f2002000000000000000000000000000000000000000000000000000000000000000000000000000000";

    #[test]
    fn test_encode() {
        let index = get_test_block_index(0);
        let expected = hex::decode(SERIALIZED_GENESIS_BLOCK_INDEX).unwrap();
        assert_eq!(serialize(&index), expected);
    }

    #[test]
    fn test_decode() {
        let index: BlockIndex =
            deserialize(&hex::decode(SERIALIZED_GENESIS_BLOCK_INDEX).unwrap()).unwrap();
        let expected = get_test_block_index(0);
        assert_eq!(index, expected);
    }
}
