// Copyright (c) 2019 Chaintope Inc.
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

use crate::chain::{BlockIndex, BlockValidationErrorCause, Error};
use crate::network::time::get_adjusted_time;
use bitcoin_hashes::{sha256d, Hash};
use core::cmp;
use hex;
use std::time::{Duration, SystemTime};
use tapyrus::{BitcoinHash, Block, BlockHeader};

/// The block version value must be in block header version field.
const BLOCK_VERSION: u32 = 1;

/// This will remove after Tapyrus core version bit is fixed as `1`.
const BIP9_VERSION_BITS_TOP_BITS: u32 = 0x20000000;

const MEDIAN_TIME_SPAN: usize = 11;

/// Maximum amount of time that a block timestamp is allowed to exceed the current network-adjusted
/// time before the block will be accepted. (seconds)
const MAX_FUTURE_BLOCK_TIME: u64 = 2 * 60 * 60;

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

impl<'a, T: ChainStore> Chain<T> {
    pub fn get_iterator(&'a self) -> ChainIterator<'a, T> {
        self.get_iterator_with_range(&self.genesis(), &self.tip())
    }

    pub fn get_iterator_with_range(
        &'a self,
        start: &BlockIndex,
        end: &BlockIndex,
    ) -> ChainIterator<'a, T> {
        assert!(start.height <= end.height);
        assert!(self.includes(start));
        assert!(self.includes(end));

        ChainIterator {
            chain: self,
            next: start.height,
            next_back: end.height,
        }
    }

    pub fn includes(&self, index: &BlockIndex) -> bool {
        match self.get(index.height) {
            Some(i) => i == *index,
            None => false,
        }
    }

    pub fn get_median_time_past(&self, index: &BlockIndex) -> u32 {
        let mut vec: Vec<u32> = self
            .get_iterator_with_range(&self.genesis(), index)
            .rev()
            .take(MEDIAN_TIME_SPAN)
            .map(|i| i.header.time)
            .collect::<Vec<u32>>();
        vec.sort();

        vec[vec.len() / 2]
    }

    /// validate block header and connect to chain tip.
    pub fn connect_block_header(&mut self, header: BlockHeader) -> Result<(), Error> {
        self.check_connect_to_tip(&header)?;
        self.connect_block_header_without_check(header);
        Ok(())
    }

    fn check_connect_to_tip(&self, header: &BlockHeader) -> Result<(), Error> {
        let tip = self.store.tip();

        // check version
        if header.version != 1 && header.version != BIP9_VERSION_BITS_TOP_BITS {
            return Err(Error::BlockValidationError(
                BlockValidationErrorCause::WrongBlockVersion {
                    wrong_version: header.version,
                    correct_version: BLOCK_VERSION,
                },
            ));
        }

        // check timestamp
        if header.time <= self.get_median_time_past(&tip) {
            return Err(Error::BlockValidationError(
                BlockValidationErrorCause::BlockTimeTooOld,
            ));
        }
        if header.time > (get_adjusted_time() + MAX_FUTURE_BLOCK_TIME) as u32 {
            return Err(Error::BlockValidationError(
                BlockValidationErrorCause::BlockTimeTooNew,
            ));
        }

        // check prev_blockhash
        if tip.header.bitcoin_hash() != header.prev_blockhash {
            return Err(Error::BlockValidationError(
                BlockValidationErrorCause::CantConnectToTip,
            ));
        }

        Ok(())
    }

    fn connect_block_header_without_check(&mut self, header: BlockHeader) {
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

    /// Return genesis block
    pub fn genesis(&self) -> BlockIndex {
        self.get(0).unwrap()
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

/// This is iterator of Chain struct for to deal continuous blocks which are part of the chain.
pub struct ChainIterator<'a, T>
where
    T: ChainStore,
{
    chain: &'a Chain<T>,
    next: i32,
    next_back: i32,
}

impl<'a, T: ChainStore> Iterator for ChainIterator<'a, T> {
    type Item = BlockIndex;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next > self.next_back {
            None
        } else {
            let old = self.chain.get(self.next).unwrap();
            self.next = self.next + 1;
            Some(old)
        }
    }
}

impl<'a, T: ChainStore> DoubleEndedIterator for ChainIterator<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.next > self.next_back {
            None
        } else {
            let old = self.chain.get(self.next_back).unwrap();
            self.next_back = self.next_back - 1;
            Some(old)
        }
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
    use crate::network::time::{now, set_mock_time};
    use crate::test_helper::{get_chain, get_test_block_hash, get_test_chain, get_test_headers};
    use std::time::{SystemTime, UNIX_EPOCH};
    use tapyrus::blockdata::block::Signature;
    use tapyrus::consensus::serialize;
    use tapyrus::Script;

    #[test]
    fn test_get_median_time_past() {
        let mut chain = get_chain();
        let mut time = chain.genesis().header.time;

        for height in 1..11 {
            chain
                .connect_block_header(BlockHeader {
                    version: 1,
                    prev_blockhash: chain.get(height - 1).unwrap().header.bitcoin_hash(),
                    merkle_root: Default::default(),
                    im_merkle_root: Default::default(),
                    time: time + 1,
                    proof: Signature {
                        signature: Script::new(),
                    },
                })
                .unwrap();

            time += 1;
        }

        assert_eq!(
            chain.get_median_time_past(&chain.get(0).unwrap()),
            chain.get(0).unwrap().header.time
        );

        assert_eq!(
            chain.get_median_time_past(&chain.get(1).unwrap()),
            chain.get(1).unwrap().header.time
        );

        assert_eq!(
            chain.get_median_time_past(&chain.get(10).unwrap()),
            chain.get(5).unwrap().header.time
        );
    }

    #[test]
    fn test_block_index_serialize() {
        let chain = get_test_chain(0);
        let index = chain.get(0).unwrap();
        let bytes = serialize(&index);

        println!("{:?}", bytes);
    }

    #[test]
    fn test_connect_block_header_set_next_blockhash() {
        let mut chain = get_test_chain(0);
        let header = get_test_headers(1, 1).pop().unwrap();
        let hash = header.bitcoin_hash();

        let _ = chain.connect_block_header(header);
        assert_eq!(chain.get(0).unwrap().next_blockhash, hash);
    }

    #[test]
    fn test_connect_block_header_fail_when_passed_connectable_block() {
        let mut chain = get_test_chain(10);

        let header = get_test_headers(11, 1).pop().unwrap();
        assert!(chain.connect_block_header(header).is_ok());

        let header = get_test_headers(13, 1).pop().unwrap();
        assert!(chain.connect_block_header(header).is_err());
    }

    #[test]
    fn test_connect_block_header_fail_when_passed_wrong_version_block() {
        let mut chain = get_test_chain(10);

        let mut header = get_test_headers(11, 1).pop().unwrap();
        header.version = 2;
        assert!(chain.connect_block_header(header).is_err());
    }

    #[test]
    fn test_check_connect_to_tip_fail_when_passed_wrong_time_block() {
        let chain = get_test_chain(10);
        let mut header = get_test_headers(11, 1).pop().unwrap();

        let now = chain.tip().header.time;
        set_mock_time(now as u64);

        // over max future
        header.time = now + MAX_FUTURE_BLOCK_TIME as u32 + 1;
        assert!(chain.check_connect_to_tip(&header).is_err());

        // 1 sec before from max future
        header.time = now + MAX_FUTURE_BLOCK_TIME as u32 - 1;
        assert!(chain.check_connect_to_tip(&header).is_ok());

        // 1 sec before from median time past
        header.time = chain.get_median_time_past(&chain.tip()) - 1;
        assert!(chain.check_connect_to_tip(&header).is_err());

        // same with median time past
        header.time = chain.get_median_time_past(&chain.tip());
        assert!(chain.check_connect_to_tip(&header).is_err());

        // 1 sec after from median time past
        header.time = chain.get_median_time_past(&chain.tip()) + 1;
        assert!(chain.check_connect_to_tip(&header).is_ok());
    }

    #[test]
    fn test_get_locator() {
        // when chain size is 1
        let chain = get_test_chain(0);
        assert_eq!(chain.get_locator(), vec![get_test_block_hash(0)]);

        // when chain size is 10
        let chain = get_test_chain(9);
        let expected: Vec<sha256d::Hash> = get_test_headers(0, 10)
            .into_iter()
            .rev()
            .map(|v| v.bitcoin_hash())
            .collect();
        assert_eq!(chain.get_locator(), expected);

        // when chain size is 100
        let chain = get_test_chain(99);
        let mut expected = Vec::new();

        for i in &[
            99, 98, 97, 96, 95, 94, 93, 92, 91, 90, 89, 88, 86, 82, 74, 58, 26, 0,
        ] {
            expected.push(get_test_block_hash(*i as usize));
        }
        assert_eq!(chain.get_locator(), expected);
    }
}
