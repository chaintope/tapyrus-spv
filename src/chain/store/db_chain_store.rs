use crate::chain::{BlockIndex, ChainStore, Error};
use bitcoin::consensus::{deserialize, serialize};
use bitcoin::BitcoinHash;
use bitcoin_hashes::{sha256d, Hash};
use rocksdb::WriteBatch;

/// BlockIndex Database
/// You can persist block headers storing it into storage with RocksDB. It only deal with only
/// single chain so far. If you try to store another fork chain into same DB, the DB indexes are
/// going to be broken.
#[derive(Debug)]
pub struct DBChainStore {
    db: rocksdb::DB,
}

/// key: block hash, value: BlockIndex
const KEY_PREFIX_ENTRY: u8 = 1;
/// key: block height in active chain, value: block hash
const KEY_PREFIX_HEIGHT: u8 = 2;
/// key: KEY_PREFIX_TIP, value: tip block hash
const KEY_PREFIX_TIP: u8 = 3;

fn entry_key(blockhash: &sha256d::Hash) -> Vec<u8> {
    let mut key = [0u8; 33];
    key[0] = KEY_PREFIX_ENTRY;
    key[1..].copy_from_slice(&blockhash.into_inner());
    key.to_vec()
}

fn height_key(height: i32) -> Vec<u8> {
    let mut key = [0u8; 5];
    key[0] = KEY_PREFIX_HEIGHT;
    key[1..].copy_from_slice(&serialize(&height));
    key.to_vec()
}

fn tip_key() -> Vec<u8> {
    vec![KEY_PREFIX_TIP]
}

impl ChainStore for DBChainStore {
    fn height(&self) -> i32 {
        self.tip().height
    }

    fn get(&self, height: i32) -> Option<BlockIndex> {
        match self.get_by_height(height) {
            Ok(r) => r,
            Err(e) => panic!("{:?}", e),
        }
    }

    fn update_tip(&mut self, index: &BlockIndex) {
        if let Err(e) = self.process_update_tip(&index) {
            panic!("{:?}", e);
        }
    }

    fn tip(&self) -> BlockIndex {
        match self.get_tip() {
            Ok(Some(index)) => index,
            Ok(None) => panic!("Tip block should be set with initialize() method before use Store"),
            Err(e) => panic!("{:?}", e),
        }
    }
}

impl DBChainStore {
    pub fn new(db: rocksdb::DB) -> DBChainStore {
        DBChainStore { db }
    }

    fn process_update_tip(&self, index: &BlockIndex) -> Result<(), Error> {
        let mut batch = WriteBatch::default();

        // update prev tip
        match self.get_tip()? {
            Some(mut prev) => {
                prev.next_blockhash = index.header.bitcoin_hash();

                let ser_hash = serialize(&prev.header.bitcoin_hash());
                let ser_index = serialize(&prev);
                batch.put(entry_key(&prev.header.bitcoin_hash()), &ser_index)?;
                batch.put(height_key(prev.height), &ser_hash)?;
            }
            None => {}
        };

        // put new tip
        let ser_hash = serialize(&index.header.bitcoin_hash());
        let ser_index = serialize(index);

        batch.put(entry_key(&index.header.bitcoin_hash()), &ser_index)?;
        batch.put(height_key(index.height), &ser_hash)?;
        batch.put(tip_key(), &ser_hash)?;

        self.db.write(batch)?;

        Ok(())
    }

    fn get_tip(&self) -> Result<Option<BlockIndex>, Error> {
        let value = self.db.get(&tip_key())?;

        if value.is_none() {
            return Ok(None);
        }

        let hash = sha256d::Hash::from_slice(value.unwrap().as_ref())?;
        self.get_by_blockhash(&hash)
    }

    /// Get BlockIndex from block height.
    fn get_by_height(&self, height: i32) -> Result<Option<BlockIndex>, Error> {
        let value = self.db.get(height_key(height))?;

        if value.is_none() {
            return Ok(None);
        }

        let hash = sha256d::Hash::from_slice(value.unwrap().as_ref())?;
        self.get_by_blockhash(&hash)
    }

    /// Get BlockIndex from blockhash.
    fn get_by_blockhash(&self, blockhash: &sha256d::Hash) -> Result<Option<BlockIndex>, Error> {
        let value = self.db.get(entry_key(blockhash))?;

        if let Some(value) = value {
            Ok(Some(deserialize::<BlockIndex>(&value)?))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::{get_test_block_hash, get_test_block_index};
    use bitcoin::blockdata::constants::genesis_block;
    use bitcoin::Network;
    use rand::Rng;
    use rocksdb::Options;

    fn db_path() -> String {
        let mut arr = [0u8; 8];
        rand::thread_rng().fill(&mut arr);
        let random = hex::encode(arr);

        format!("/tmp/tapyrus-spv-test/{}", random)
    }

    #[test]
    fn test_store() {
        let path = db_path();
        {
            let db = rocksdb::DB::open_default(&path).unwrap();
            let mut store = DBChainStore::new(db);
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
        let _ = rocksdb::DB::destroy(&Options::default(), &path);
    }

    #[test]
    fn test_entry_key() {
        let expected: Vec<u8> = vec![
            1, 6, 34, 110, 70, 17, 26, 11, 89, 202, 175, 18, 96, 67, 235, 91, 191, 40, 195, 79, 58,
            94, 51, 42, 31, 199, 178, 183, 60, 241, 136, 145, 15,
        ];
        assert_eq!(entry_key(&get_test_block_hash(0)), expected);
    }
}
