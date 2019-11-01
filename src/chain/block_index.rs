// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

use bitcoin_hashes::sha256d;
use tapyrus::consensus::{Decodable, Encodable};
use tapyrus::{BlockHeader, Network};

/// This struct is a index of block header. It has not only block header but also meta data like
/// 'height', 'next_blockhash' for that.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockIndex {
    pub header: BlockHeader,
    pub height: i32,
    pub next_blockhash: sha256d::Hash,
}

impl Encodable for BlockIndex {
    #[inline]
    fn consensus_encode<S: std::io::Write>(
        &self,
        mut s: S,
    ) -> Result<usize, tapyrus::consensus::encode::Error> {
        let mut len = 0;
        len += self.header.consensus_encode(&mut s)?;
        len += self.height.consensus_encode(&mut s)?;
        len += self.next_blockhash.consensus_encode(&mut s)?;
        Ok(len)
    }
}

impl Decodable for BlockIndex {
    #[inline]
    fn consensus_decode<D: std::io::Read>(
        mut d: D,
    ) -> Result<Self, tapyrus::consensus::encode::Error> {
        Ok(BlockIndex {
            header: Decodable::consensus_decode(&mut d)?,
            height: Decodable::consensus_decode(&mut d)?,
            next_blockhash: Decodable::consensus_decode(&mut d)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::get_test_block_index;
    use tapyrus::consensus::{deserialize, serialize};

    const SERIALIZED_GENESIS_BLOCK_INDEX: &str = "01000000000000000000000000000000000000000000000000000000000000000000000019225b47d8c3eefd0dab934ba4b633940d032a7f5a192a4ddece08f566f1cfb95d5022ed80bde51d7436cadcb10455a2e5523fea9e46dc9ee5dec0037387e1b137aaba5d40fd3748264662cd991ac70e8d9ae3e06a1ea8956d74b36aa6419ca428f9baf24dc4df95637dc524d6374ef59ef6d15aba25020de3c35da969b1329ec961488067000000000000000000000000000000000000000000000000000000000000000000000000";

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
