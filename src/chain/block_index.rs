use bitcoin::blockdata::constants::genesis_block;
use bitcoin::consensus::{Decodable, Decoder, Encodable, Encoder};
use bitcoin::{BlockHeader, Network};
use bitcoin_hashes::sha256d;

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
