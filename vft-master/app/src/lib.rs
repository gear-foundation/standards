#![no_std]
#![allow(clippy::new_without_default)]

use gstd::String;
use sails::gstd::gprogram;
mod services;
use services::vft_master::VftMaster;
pub struct Program(());

#[gprogram]
impl Program {
    pub fn new(name: String, symbol: String, decimals: u8) -> Self {
        VftMaster::seed(name, symbol, decimals);
        Self(())
    }

    pub fn vft(&self) -> VftMaster {
        VftMaster::new()
    }
}
