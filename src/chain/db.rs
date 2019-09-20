use crate::chain::{BlockIndex, Error};
use bitcoin::consensus::{deserialize, serialize};
use bitcoin::BitcoinHash;
use bitcoin_hashes::{sha256d, Hash};
use rocksdb::WriteBatch;

/// BlockIndex Database
pub struct DB {
    db: rocksdb::DB,
}

impl DB {
    pub fn new(path: &str) -> DB {
        DB {
            db: rocksdb::DB::open_default(path).unwrap(),
        }
    }

    /// Put BlockIndex into Database.
    pub fn put(&self, index: &BlockIndex) -> Result<(), Error> {
        let index = index.clone();
        let blockhash = index.header.bitcoin_hash().into_inner();
        let value = serialize(&index);
        let serialized_height = serialize(&index.height);

        let mut batch = WriteBatch::default();
        batch.put(&blockhash, &value)?;
        batch.put(serialized_height, &blockhash)?;
        self.db.write(batch)?;

        Ok(())
    }

    /// Get BlockIndex from block height.
    pub fn get(&self, height: u32) -> Result<Option<BlockIndex>, Error> {
        let value = self.db.get(serialize(&height))?;

        if value.is_none() {
            return Ok(None);
        }

        let hash = sha256d::Hash::from_slice(value.unwrap().as_ref())?;
        self.get_by_blockhash(&hash)
    }

    /// Get BlockIndex from blockhash.
    pub fn get_by_blockhash(&self, blockhash: &sha256d::Hash) -> Result<Option<BlockIndex>, Error> {
        let value = self.db.get(serialize(blockhash))?;

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
    use bitcoin::blockdata::constants::genesis_block;
    use bitcoin::Network;
    use bitcoin_hashes::sha256d;
    use rand::Rng;
    use rocksdb::Options;

    fn db_path() -> String {
        let mut arr = [0u8; 8];
        rand::thread_rng().fill(&mut arr);
        let random = hex::encode(arr);

        format!("/tmp/tapyrus-spv-test/{}", random)
    }

    #[test]
    fn test_put() {
        let path = db_path();
        {
            let db = DB::new(&path);

            let index = BlockIndex {
                header: genesis_block(Network::Regtest).header,
                height: 0,
                next_blockhash: sha256d::Hash::default(),
            };

            assert!(db.put(&index).is_ok());
            {
                let value = db.db.get(serialize(&0_u32)).unwrap().unwrap();
                let blockhash = deserialize::<sha256d::Hash>(&value).unwrap();
                assert_eq!(blockhash, index.header.bitcoin_hash());
            }

            {
                let blockhash = index.header.bitcoin_hash();
                let value = db.db.get(serialize(&blockhash)).unwrap().unwrap();
                let index = deserialize::<BlockIndex>(&value).unwrap();
                assert_eq!(index, index);
            }
        }

        let _ = rocksdb::DB::destroy(&Options::default(), &path);
    }

    #[test]
    fn test_get() {
        let path = db_path();

        {
            let db = DB::new(&path);

            assert!(db.get(0).unwrap().is_none());

            let index = BlockIndex {
                header: genesis_block(Network::Regtest).header,
                height: 0,
                next_blockhash: sha256d::Hash::default(),
            };

            assert!(db.put(&index).is_ok());
            assert_eq!(db.get(0).unwrap().unwrap(), index);
        }

        let _ = rocksdb::DB::destroy(&Options::default(), &path);
    }

    #[test]
    fn test_get_by_blockhash() {
        let path = db_path();

        {
            let db = DB::new(&path);

            let index = BlockIndex {
                header: genesis_block(Network::Regtest).header,
                height: 0,
                next_blockhash: sha256d::Hash::default(),
            };

            assert!(db
                .get_by_blockhash(&index.header.bitcoin_hash())
                .unwrap()
                .is_none());

            assert!(db.put(&index).is_ok());
            assert_eq!(
                db.get_by_blockhash(&index.header.bitcoin_hash())
                    .unwrap()
                    .unwrap(),
                index
            );
        }

        let _ = rocksdb::DB::destroy(&Options::default(), &path);
    }
}
