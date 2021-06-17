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
        let genesis_raw = "010000000000000000000000000000000000000000000000000000000000000000000000623fd6e71aaec98e129d8b447ba7c6fe88cd27346cc556353d2d8232a2829f0a49b4a19f4dc3f0526dca905dcaff6a8e34537d04b450e0ac5568ce89a9373e301665c860012102260b9be70a87125fd0e2da368db857a2d8ee1cb85a3c8b81490f4f35f99b212a40f457d5dd7caae6bf89a50efd13cf4a9e3857760f747e107d1184263e1212ef98cda52410bd822e92c44b4a22a2f6116a0df2b130afe315af6a7289567b32c1ca01010000000100000000000000000000000000000000000000000000000000000000000000000000000000ffffffff0100f2052a010000002776a9226d6b597162714c54584e52344568747853376e37734e5357385646546f314e55336e88ac00000000";
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
