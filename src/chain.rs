use bitcoin::{BlockHeader, Network, BitcoinHash};
use bitcoin::blockdata::constants::genesis_block;
use bitcoin_hashes::sha256d;

pub struct Error;

pub struct ChainState {
    chain_active: Chain
}

impl ChainState {
    pub fn new() -> ChainState {
        let genesis = genesis_block(Network::Regtest);
        ChainState {
            chain_active: Chain {
                headers: vec![genesis.header]
            }
        }
    }

    pub fn borrow_chain_active(& self) -> &Chain {
        &self.chain_active
    }

    pub fn borrow_mut_chain_active(&mut self) -> &mut Chain {
        &mut self.chain_active
    }
}

pub struct Chain {
    headers: Vec<BlockHeader>
}

impl Chain {
    /// validate block header and connect to chain tip.
    /// TODO: implement validation
    pub fn connect_block_header(&mut self, header: BlockHeader) -> Result<(), Error> {
        self.headers.push(header);
        Ok(())
    }

    pub fn height(&self) -> usize {
        self.headers.len() - 1
    }

    pub fn get_locator(&self) -> Vec<sha256d::Hash> {
        let genesis = genesis_block(Network::Regtest);
        vec![genesis.header.bitcoin_hash()]
    }
}