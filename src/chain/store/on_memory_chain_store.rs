// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

use crate::chain::{BlockIndex, ChainStore};
use tapyrus::{Block, BlockHash};

pub struct OnMemoryChainStore {
    headers: Vec<BlockIndex>,
}

impl ChainStore for OnMemoryChainStore {
    fn initialize(&mut self, genesis: Block) {
        if let None = self.get(0) {
            let genesis = BlockIndex {
                header: genesis.header,
                height: 0,
                next_blockhash: BlockHash::default(),
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
        tip.next_blockhash = index.header.block_hash();

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
    use tapyrus::consensus::deserialize;

    #[test]
    fn test_store() {
        let mut store = OnMemoryChainStore::new();
        let genesis_raw = "01000000000000000000000000000000000000000000000000000000000000000000000019225b47d8c3eefd0dab934ba4b633940d032a7f5a192a4ddece08f566f1cfb95d5022ed80bde51d7436cadcb10455a2e5523fea9e46dc9ee5dec0037387e1b137aaba5d40fd3748264662cd991ac70e8d9ae3e06a1ea8956d74b36aa6419ca428f9baf24dc4df95637dc524d6374ef59ef6d15aba25020de3c35da969b1329ec961488067010000002001000000000000000000000000000000000000000000000000000000000000000000000000222102260b9be70a87125fd0e2da368db857a2d8ee1cb85a3c8b81490f4f35f99b212affffffff0100f2052a010000001976a9143733df9979ee67615b16aff5b210d894557325df88ac00000000";
        let genesis_block = deserialize(&hex::decode(genesis_raw).unwrap()).unwrap();
        store.initialize(genesis_block);

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
        expected.next_blockhash = get_test_block_index(4).header.block_hash();
        assert_eq!(store.get(3), Some(expected));
    }
}
