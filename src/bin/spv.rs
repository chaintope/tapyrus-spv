extern crate simple_logger;

use bitcoin::network::constants::Network;
use log::Level;
use tapyrus_spv::SPV;

fn main() {
    // TODO: specify log level by user argument
    simple_logger::init_with_level(Level::Trace).unwrap();

    let spv = SPV::new(Network::Regtest);
    spv.run();
}
