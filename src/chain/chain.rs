// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

use crate::chain::{BlockIndex, Error};
use tapyrus::{BitcoinHash, Block, BlockHeader};
use bitcoin_hashes::{sha256d, Hash};
use core::cmp;
use hex;

/// This struct presents the way to use single chain.
#[derive(Debug)]
pub struct Chain<T>
where
    T: ChainStore,
{
    store: T,
}

impl<T: ChainStore> Chain<T> {
    pub fn new(store: T) -> Chain<T> {
        Chain { store }
    }
}

impl<T: ChainStore> Chain<T> {
    // validate block header and connect to chain tip.
    // TODO: implement validation
    pub fn connect_block_header(&mut self, header: BlockHeader) -> Result<(), Error> {
        let block_index = BlockIndex {
            header,
            height: self.height() + 1,
            next_blockhash: sha256d::Hash::default(),
        };

        if log_enabled!(log::Level::Trace) {
            let hash = hex::encode(block_index.header.bitcoin_hash().into_inner());
            trace!(
                "Connect new block to tip. height: {}, hash: {}",
                block_index.height,
                hash
            );
        }

        self.store.update_tip(&block_index);

        Ok(())
    }

    /// Return height of tip.
    pub fn height(&self) -> i32 {
        self.store.height()
    }

    /// Return specific block which is indicated by height.
    pub fn get(&self, height: i32) -> Option<BlockIndex> {
        self.store.get(height)
    }

    /// Return latest block in this chain.
    pub fn tip(&self) -> BlockIndex {
        // Genesis block always exist, so we can call unwrap()
        self.get(self.height()).unwrap()
    }

    /// Return block hash list for indicate which blocks are include in block.
    pub fn get_locator(&self) -> Vec<sha256d::Hash> {
        let mut step: i32 = 1;
        let mut have = Vec::<sha256d::Hash>::with_capacity(32);

        let mut index = self.tip();

        loop {
            have.push(index.header.bitcoin_hash());

            // Stop when we have added the genesis block.
            if index.height == 0 {
                break;
            }

            let height = cmp::max(index.height - step, 0);

            // TODO: Add implementation for forked chain.
            // Check this chain contains `index`. If it is true, update `index` like below. If it is
            // not true, we should explore ancestor block and set to 'index'.
            index = self.get(height).unwrap();

            if have.len() > 10 {
                step *= 2;
            }
        }

        have
    }
}

/// This is a trait which presents interfaces to access block headers storing anywhere (e.g. on
/// memory, flash).
pub trait ChainStore {
    /// Initialize chain store.
    /// This method should be called before start to use store.
    ///
    /// ## implemnt
    /// You should implement process which should be done before use store such as setting genesis
    /// block.
    fn initialize(&mut self, genesis: Block) {
        if let None = self.get(0) {
            let genesis = BlockIndex {
                header: genesis.header,
                height: 0,
                next_blockhash: sha256d::Hash::default(),
            };

            self.update_tip(&genesis);
        }
    }

    /// Return height of tip.
    fn height(&self) -> i32;

    /// Return specific block which is indicated by height.
    fn get(&self, height: i32) -> Option<BlockIndex>;

    /// Update chain tip to passed BlockIndex.
    fn update_tip(&mut self, index: &BlockIndex);

    /// Return latest block in this chain.
    fn tip(&self) -> BlockIndex {
        // Genesis block always exist, so we can call unwrap()
        self.get(self.height()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::store::OnMemoryChainStore;
    use crate::test_helper::{get_test_block_hash, get_test_headers, get_chain};
    use tapyrus::consensus::serialize;
    use tapyrus::Network;

    fn build_chain(height: usize) -> Chain<OnMemoryChainStore> {
        let mut chain = get_chain();

        if height == 0 {
            return chain;
        }

        for header in get_test_headers(1, height) {
            let _ = chain.connect_block_header(header);
        }

        chain
    }

    #[test]
    fn test_block_index_serialize() {
        let chain = build_chain(0);
        let index = chain.get(0).unwrap();
        let bytes = serialize(&index);

        println!("{:?}", bytes);
    }

    #[test]
    fn test_connect_block_header_set_next_blockhash() {
        let mut chain = build_chain(0);
        let header = get_test_headers(1, 1).pop().unwrap();
        let hash = header.bitcoin_hash();

        let _ = chain.connect_block_header(header);
        assert_eq!(chain.get(0).unwrap().next_blockhash, hash);
    }

    #[test]
    fn test_get_locator() {
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
