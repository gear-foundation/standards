#![no_std]
#![allow(clippy::new_without_default)]

use sails_rs::prelude::*;
mod services;
use services::extended_vft::ExtendedService;
pub struct ExtendedVftProgram(());

#[program]
impl ExtendedVftProgram {
    pub fn new(
        name: String,
        symbol: String,
        decimals: u8,
        balances_caps: Option<Vec<u32>>,
        allowances_caps: Option<Vec<u32>>,
    ) -> Self {
        ExtendedService::init(name, symbol, decimals, balances_caps, allowances_caps);
        Self(())
    }

    pub fn vft(&self) -> ExtendedService {
        ExtendedService::new()
    }
}
