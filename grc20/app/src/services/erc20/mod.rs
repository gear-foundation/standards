use core::fmt::Debug;
use gstd::{collections::HashMap, format, msg, ActorId, Decode, Encode, String, TypeInfo, Vec};
use primitive_types::U256;
use sails_rtl::gstd::gservice;

pub use utils::*;

pub mod funcs;
pub(crate) mod utils;

static mut ERC20_STORAGE: Option<Erc20Storage> = None;

#[derive(Debug, Default)]
struct Erc20Storage {
    balances: HashMap<ActorId, NonZeroU256>,
    allowances: HashMap<(ActorId, ActorId), NonZeroU256>,
    meta: Metadata,
    total_supply: U256,
}

impl Erc20Storage {
    pub fn get_mut() -> &'static mut Self {
        unsafe {
            ERC20_STORAGE
                .as_mut()
                .expect("Erc20Storage is not initialized")
        }
    }
    pub fn get() -> &'static Self {
        unsafe {
            ERC20_STORAGE
                .as_ref()
                .expect("Erc20Storage is not initialized")
        }
    }
}

#[derive(Debug, Default)]
pub struct Metadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
pub enum Event {
    Approval {
        owner: sails_rtl::ActorId,
        spender: sails_rtl::ActorId,
        value: U256,
    },
    Transfer {
        from: sails_rtl::ActorId,
        to: sails_rtl::ActorId,
        value: U256,
    },
}

#[derive(Clone)]
pub struct Service();

impl Service {
    pub fn seed(name: String, symbol: String, decimals: u8) -> Self {
        unsafe {
            ERC20_STORAGE = Some(Erc20Storage {
                meta: Metadata {
                    name,
                    symbol,
                    decimals,
                },
                ..Default::default()
            });
        }
        Self()
    }
}

#[gservice(events = Event)]
impl Service {
    pub fn new() -> Self {
        Self()
    }

    pub fn approve(&mut self, spender: sails_rtl::ActorId, value: U256) -> bool {
        let owner = msg::source();
        let storage = Erc20Storage::get_mut();
        let mutated = funcs::approve(&mut storage.allowances, owner, spender.into(), value);

        if mutated {
            let _ = self.notify_on(Event::Approval {
                owner: owner.into(),
                spender,
                value,
            });
        }

        mutated
    }

    pub fn transfer(&mut self, to: sails_rtl::ActorId, value: U256) -> bool {
        let from = msg::source();
        let storage = Erc20Storage::get_mut();
        let mutated =
            panicking(move || funcs::transfer(&mut storage.balances, from, to.into(), value));

        if mutated {
            let _ = self.notify_on(Event::Transfer {
                from: from.into(),
                to,
                value,
            });
        }

        mutated
    }

    pub fn transfer_from(
        &mut self,
        from: sails_rtl::ActorId,
        to: sails_rtl::ActorId,
        value: U256,
    ) -> bool {
        let spender = msg::source();
        let storage = Erc20Storage::get_mut();
        let mutated = panicking(move || {
            funcs::transfer_from(
                &mut storage.allowances,
                &mut storage.balances,
                spender,
                from.into(),
                to.into(),
                value,
            )
        });

        if mutated {
            let _ = self.notify_on(Event::Transfer { from, to, value });
        }

        mutated
    }

    pub fn allowance(&self, owner: sails_rtl::ActorId, spender: sails_rtl::ActorId) -> U256 {
        let storage = Erc20Storage::get();
        funcs::allowance(&storage.allowances, owner.into(), spender.into())
    }

    pub fn balance_of(&self, account: sails_rtl::ActorId) -> U256 {
        let storage = Erc20Storage::get();
        funcs::balance_of(&storage.balances, account.into())
    }

    pub fn decimals(&self) -> &'static u8 {
        let storage = Erc20Storage::get();
        &storage.meta.decimals
    }

    pub fn name(&self) -> &'static str {
        let storage = Erc20Storage::get();
        &storage.meta.name
    }

    pub fn symbol(&self) -> &'static str {
        let storage = Erc20Storage::get();
        &storage.meta.symbol
    }

    pub fn total_supply(&self) -> &'static U256 {
        let storage = Erc20Storage::get();
        &storage.total_supply
    }
}
