#![no_std]
include!(concat!(env!("OUT_DIR"), "/extended_vmt_client.rs"));
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

#[cfg(target_arch = "wasm32")]
pub use extended_vmt_app::wasm::*;
