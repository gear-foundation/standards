#![no_std]
#![allow(clippy::new_without_default)]

use gstd::String;
use sails_rtl::gstd::gprogram;
use services::erc20;

mod services;

pub struct Program(());

#[gprogram]
impl Program {
    pub fn new(name: String, symbol: String, decimals: u8) -> Self {
        <erc20::Service>::seed(name, symbol, decimals);
        Self(())
    }

    pub fn erc20(&self) -> erc20::Service {
        erc20::Service::new()
    }
}
