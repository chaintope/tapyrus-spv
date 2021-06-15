// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

use tapyrus::consensus::{Decodable, Encodable};
use tapyrus::{BlockHash, BlockHeader};

/// This struct is a index of block header. It has not only block header but also meta data like
/// 'height', 'next_blockhash' for that.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockIndex {
    pub header: BlockHeader,
    pub height: i32,
    pub next_blockhash: BlockHash,
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

    const SERIALIZED_GENESIS_BLOCK_INDEX: &str = "010000000000000000000000000000000000000000000000000000000000000000000000623fd6e71aaec98e129d8b447ba7c6fe88cd27346cc556353d2d8232a2829f0a49b4a19f4dc3f0526dca905dcaff6a8e34537d04b450e0ac5568ce89a9373e301665c860012102260b9be70a87125fd0e2da368db857a2d8ee1cb85a3c8b81490f4f35f99b212a40f457d5dd7caae6bf89a50efd13cf4a9e3857760f747e107d1184263e1212ef98cda52410bd822e92c44b4a22a2f6116a0df2b130afe315af6a7289567b32c1ca000000000000000000000000000000000000000000000000000000000000000000000000";

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
