// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

use crate::chain::{BlockIndex, ChainStore};
use bitcoin::{BitcoinHash, Block};
use bitcoin_hashes::sha256d;

pub struct OnMemoryChainStore {
    headers: Vec<BlockIndex>,
}

impl ChainStore for OnMemoryChainStore {
    fn initialize(&mut self, genesis: Block) {
        if let None = self.get(0) {
            let genesis = BlockIndex {
                header: genesis.header,
                height: 0,
                next_blockhash: sha256d::Hash::default(),
            };

            self.headers = vec![genesis];
        }
    }

    fn height(&self) -> i32 {
        self.headers.len() as i32 - 1
    }

    fn get(&self, height: i32) -> Option<BlockIndex> {
        match self.headers.get(height as usize) {
            Some(index) => Some(index.clone()),
            None => None,
        }
    }

    fn update_tip(&mut self, index: &BlockIndex) {
        let mut tip = self.tip_mut();
        tip.next_blockhash = index.header.bitcoin_hash();

        self.headers.push(index.clone());
    }
}

impl OnMemoryChainStore {
    pub fn new() -> OnMemoryChainStore {
        OnMemoryChainStore { headers: vec![] }
    }

    fn get_mut(&mut self, height: i32) -> Option<&mut BlockIndex> {
        self.headers.get_mut(height as usize)
    }

    fn tip_mut(&mut self) -> &mut BlockIndex {
        // Genesis block always exist, so we can call unwrap()
        self.get_mut(self.height()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::ChainStore;
    use crate::test_helper::get_test_block_index;
    use bitcoin::blockdata::constants::genesis_block;
    use bitcoin::{BitcoinHash, Network};

    #[test]
    fn test_store() {
        let mut store = OnMemoryChainStore::new();
        store.initialize(genesis_block(Network::Regtest));

        assert!(store.get(0).is_some());
        assert_eq!(store.height(), 0);

        // test update_tip
        store.update_tip(&get_test_block_index(1));
        assert_eq!(store.height(), 1);
        assert_eq!(store.tip(), get_test_block_index(1));

        // update tip to 10
        for i in 2..11 {
            store.update_tip(&get_test_block_index(i));
        }
        assert_eq!(store.height(), 10);
        assert_eq!(store.tip(), get_test_block_index(10));

        // test get()
        let mut expected = get_test_block_index(3);
        expected.next_blockhash = get_test_block_index(4).header.bitcoin_hash();
        assert_eq!(store.get(3), Some(expected));
    }
}
