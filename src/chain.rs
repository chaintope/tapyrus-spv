use bitcoin::blockdata::constants::genesis_block;
use bitcoin::{BitcoinHash, BlockHeader, Network};
use bitcoin_hashes::{sha256d, Hash};
use core::cmp;
use hex;

pub struct Error;

pub struct ChainState {
    chain_active: Chain,
}

impl ChainState {
    pub fn new() -> ChainState {
        ChainState {
            chain_active: Chain::default(),
        }
    }

    pub fn borrow_chain_active(&self) -> &Chain {
        &self.chain_active
    }

    pub fn borrow_mut_chain_active(&mut self) -> &mut Chain {
        &mut self.chain_active
    }
}

#[derive(Debug)]
pub struct BlockIndex {
    pub header: BlockHeader,
    pub height: usize,
}

#[derive(Debug)]
pub struct Chain {
    headers: Vec<BlockIndex>,
}

impl Chain {
    /// validate block header and connect to chain tip.
    /// TODO: implement validation
    pub fn connect_block_header(&mut self, header: BlockHeader) -> Result<(), Error> {
        let block_index = BlockIndex {
            header,
            height: self.height() + 1,
        };

        if log_enabled!(log::Level::Trace) {
            let hash = hex::encode(block_index.header.bitcoin_hash().into_inner());
            trace!(
                "Connect new block to tip. height: {}, hash: {}",
                block_index.height,
                hash
            );
        }

        self.headers.push(block_index);
        Ok(())
    }

    /// Return height of tip.
    pub fn height(&self) -> usize {
        self.headers.len() - 1
    }

    /// Return specific block which is indicated by height.
    pub fn get(&self, height: usize) -> Option<&BlockIndex> {
        self.headers.get(height)
    }

    /// Return latest block in this chain.
    pub fn tip(&self) -> &BlockIndex {
        // Genesis block always exist, so we can call unwrap()
        self.get(self.height()).unwrap()
    }

    /// Return block hash list for indicate which blocks are include in block.
    pub fn get_locator(&self) -> Vec<sha256d::Hash> {
        let mut step: isize = 1;
        let mut have = Vec::<sha256d::Hash>::with_capacity(32);

        let mut index = self.tip();

        loop {
            have.push(index.header.bitcoin_hash());

            // Stop when we have added the genesis block.
            if index.height == 0 {
                break;
            }

            let height = cmp::max(index.height as isize - step, 0);

            // TODO: Add implementation for forked chain.
            // Check this chain contains `index`. If it is true, update `index` like below. If it is
            // not true, we should explore ancestor block and set to 'index'.
            index = &self.headers[height as usize];

            if have.len() > 10 {
                step *= 2;
            }
        }

        have
    }
}

impl Default for Chain {
    fn default() -> Self {
        let index = BlockIndex {
            header: genesis_block(Network::Regtest).header,
            height: 0,
        };
        Chain {
            headers: vec![index],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::{get_test_block_hash, get_test_headers};

    fn build_chain(height: usize) -> Chain {
        let mut chain = Chain::default();

        if height == 0 {
            return chain;
        }

        for header in get_test_headers(1, height) {
            let _ = chain.connect_block_header(header);
        }

        chain
    }

    #[test]
    fn get_locator_test() {
        // when chain size is 1
        let chain = build_chain(0);
        assert_eq!(chain.get_locator(), vec![get_test_block_hash(0)]);

        // when chain size is 10
        let chain = build_chain(9);
        let expected: Vec<sha256d::Hash> = get_test_headers(0, 10)
            .into_iter()
            .rev()
            .map(|v| v.bitcoin_hash())
            .collect();
        assert_eq!(chain.get_locator(), expected);

        // when chain size is 100
        let chain = build_chain(99);
        let mut expected = Vec::new();

        for i in &[
            99, 98, 97, 96, 95, 94, 93, 92, 91, 90, 89, 88, 86, 82, 74, 58, 26, 0,
        ] {
            expected.push(get_test_block_hash(*i as usize));
        }
        assert_eq!(chain.get_locator(), expected);
    }
}
