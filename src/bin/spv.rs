extern crate simple_logger;

use tapyrus_spv::SPV;
use log::Level;

fn main() {
    // TODO: specify log level by user argument
    simple_logger::init_with_level(Level::Trace).unwrap();

    let spv = SPV::new();
    spv.run();
}
