#![no_std]
#![allow(clippy::new_without_default)]

use sails_rs::prelude::*;
mod services;
use services::extended_vmt::ExtendedService;
pub struct ExtendedVmtProgram(());

#[program]
impl ExtendedVmtProgram {
    pub fn new(name: String, symbol: String, decimals: u8) -> Self {
        ExtendedService::seed(name, symbol, decimals);
        Self(())
    }

    pub fn vmt(&self) -> ExtendedService {
        ExtendedService::new()
    }
}
