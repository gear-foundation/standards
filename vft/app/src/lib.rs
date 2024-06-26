#![no_std]
#![allow(clippy::new_without_default)]

use gstd::String;
use sails_rtl::gstd::gprogram;
use services::vft;

mod services;

pub struct Program(());

#[gprogram]
impl Program {
    pub fn new(name: String, symbol: String, decimals: u8) -> Self {
        <vft::Service>::seed(name, symbol, decimals);
        Self(())
    }

    pub fn vft(&self) -> vft::Service {
        vft::Service::new()
    }
}
