extern crate simple_logger;

use tapyrus_spv::SPV;
use log::Level;
use bitcoin::network::constants::Network;

fn main() {
    // TODO: specify log level by user argument
    simple_logger::init_with_level(Level::Trace).unwrap();

    let spv = SPV::new(Network::Regtest);
    spv.run();
}
