#![no_std]
#![allow(clippy::new_without_default)]
#![allow(unused_imports)]
#![allow(static_mut_refs)]
use core::fmt::Debug;
use sails_rs::{
    collections::HashMap,
    gstd::{msg, service},
    prelude::*,
};

pub mod funcs;
pub mod utils;

static mut STORAGE: Option<Storage> = None;

#[derive(Debug, Default)]
pub struct Storage {
    balances: HashMap<ActorId, U256>,
    allowances: HashMap<(ActorId, ActorId), U256>,
    meta: Metadata,
    total_supply: U256,
}

impl Storage {
    pub fn get_mut() -> &'static mut Self {
        unsafe { STORAGE.as_mut().expect("Storage is not initialized") }
    }
    pub fn get() -> &'static Self {
        unsafe { STORAGE.as_ref().expect("Storage is not initialized") }
    }
    pub fn balances() -> &'static mut HashMap<ActorId, U256> {
        let storage = unsafe { STORAGE.as_mut().expect("Storage is not initialized") };
        &mut storage.balances
    }
    pub fn total_supply() -> &'static mut U256 {
        let storage = unsafe { STORAGE.as_mut().expect("Storage is not initialized") };
        &mut storage.total_supply
    }
}

#[derive(Debug, Default)]
pub struct Metadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

#[event]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
pub enum Event {
    Approval {
        owner: ActorId,
        spender: ActorId,
        value: U256,
    },
    Transfer {
        from: ActorId,
        to: ActorId,
        value: U256,
    },
}

#[derive(Clone)]
pub struct Service;

impl Service {
    pub fn new() -> Self {
        Self
    }

    pub fn init(name: String, symbol: String, decimals: u8) -> Self {
        unsafe {
            STORAGE = Some(Storage {
                meta: Metadata {
                    name,
                    symbol,
                    decimals,
                },
                ..Default::default()
            });
        }
        Self
    }
}

#[service(events = Event)]
impl Service {
    #[export]
    pub fn approve(&mut self, spender: ActorId, value: U256) -> bool {
        let owner = msg::source();
        let storage = Storage::get_mut();
        let mutated = funcs::approve(&mut storage.allowances, owner, spender, value);

        if mutated {
            self.emit_event(Event::Approval {
                owner,
                spender,
                value,
            })
            .expect("Notification Error");
        }

        mutated
    }

    #[export]
    pub fn transfer(&mut self, to: ActorId, value: U256) -> bool {
        let from = msg::source();
        let storage = Storage::get_mut();
        let mutated =
            utils::panicking(move || funcs::transfer(&mut storage.balances, from, to, value));

        if mutated {
            self.emit_event(Event::Transfer { from, to, value })
                .expect("Notification Error");
        }

        mutated
    }

    #[export]
    pub fn transfer_from(&mut self, from: ActorId, to: ActorId, value: U256) -> bool {
        let spender = msg::source();
        let storage = Storage::get_mut();
        let mutated = utils::panicking(move || {
            funcs::transfer_from(
                &mut storage.allowances,
                &mut storage.balances,
                spender,
                from,
                to,
                value,
            )
        });

        if mutated {
            self.emit_event(Event::Transfer { from, to, value })
                .expect("Notification Error");
        }

        mutated
    }

    #[export]
    pub fn allowance(&self, owner: ActorId, spender: ActorId) -> U256 {
        let storage = Storage::get();
        funcs::allowance(&storage.allowances, owner, spender)
    }

    #[export]
    pub fn balance_of(&self, account: ActorId) -> U256 {
        let storage = Storage::get();
        funcs::balance_of(&storage.balances, account)
    }

    #[export]
    pub fn decimals(&self) -> &'static u8 {
        let storage = Storage::get();
        &storage.meta.decimals
    }

    #[export]
    pub fn name(&self) -> &'static str {
        let storage = Storage::get();
        &storage.meta.name
    }

    #[export]
    pub fn symbol(&self) -> &'static str {
        let storage = Storage::get();
        &storage.meta.symbol
    }

    #[export]
    pub fn total_supply(&self) -> &'static U256 {
        let storage = Storage::get();
        &storage.total_supply
    }
}
