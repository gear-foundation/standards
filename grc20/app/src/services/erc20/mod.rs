#![allow(clippy::unused_unit)]

use crate::services;
use core::{cmp::Ordering, fmt::Debug, marker::PhantomData};
use gstd::{ext, format, msg, ActorId, Decode, Encode, String, TypeInfo, Vec};
use primitive_types::U256;
use sails_rtl::gstd::gservice;
use storage::{AllowancesStorage, BalancesStorage, MetaStorage, TotalSupplyStorage};

pub use utils::*;

pub mod funcs;
pub mod storage;
pub(crate) mod utils;

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
        // TODO (breathx and team): use or not to use `NonZeroU256`?
        value: U256,
    },
}

pub type GstdDrivenService = Service;

// TODO (sails): isn't services - modules?
#[derive(Clone)]
pub struct Service();

impl Service {
    pub fn seed(name: String, symbol: String, decimals: u8) -> Self {
        let _res = AllowancesStorage::default();
        debug_assert!(_res.is_ok());

        let _res = BalancesStorage::default();
        debug_assert!(_res.is_ok());

        let _res = MetaStorage::with_data(name, symbol, decimals);
        debug_assert!(_res.is_ok());

        let _res = TotalSupplyStorage::default();
        debug_assert!(_res.is_ok());

        Self()
    }
}

// TODO (sails): consider renaming `EventTrigger` -> `Notifier`/`Informer`.
// TODO (sails): fix that requires `Encode`, `Decode`, `TypeInfo` and `Vec` in scope.
// TODO (sails): fix that requires explicit `-> ()`. ALREADY EXISTS
// TODO (sails): let me specify error as subset of strings (Display of my Error) -> thats common flow for us.
// TODO (sails): gstd::ActorId, primitive_types::H256/U256, [u8; 32], NonZeroStuff are primitives!.
// TODO (sails): gservice(events = Event, error = Error)
// #[gservice(events = Event, error = Error)]
#[gservice(events = Event)]
impl Service {
    // TODO (sails): hide this into macro.
    pub fn new() -> Self {
        Self()
    }

    pub fn approve(&mut self, spender: sails_rtl::ActorId, value: U256) -> bool {
        let owner = msg::source();

        let mutated = funcs::approve(AllowancesStorage::as_mut(), owner, spender.into(), value);

        if mutated {
            self.notify_on(Event::Approval {
                    owner: owner.into(),
                    spender,
                    value,
                });
        }

        mutated
    }

    pub fn transfer(&mut self, to: sails_rtl::ActorId, value: U256) -> bool {
        let from = msg::source();

        let mutated = services::utils::panicking(move || {
            funcs::transfer(BalancesStorage::as_mut(), from, to.into(), value)
        });

        if mutated {
            self.notify_on(Event::Transfer {
                from: from.into(),
                to,
                value,
            });
            
        }

        mutated
    }

    // TODO (breathx): rename me once bug in sails fixed.
    pub fn transfer_from(
        &mut self,
        from: sails_rtl::ActorId,
        to: sails_rtl::ActorId,
        value: U256,
    ) -> bool {
        let spender = msg::source();

        let mutated = services::utils::panicking(move || {
            funcs::transfer_from(
                AllowancesStorage::as_mut(),
                BalancesStorage::as_mut(),
                spender,
                from.into(),
                to.into(),
                value,
            )
        });

        if mutated {
            self.notify_on(Event::Transfer { from, to, value });
        }

        mutated
    }

    pub fn allowance(&self, owner: sails_rtl::ActorId, spender: sails_rtl::ActorId) -> U256 {
        funcs::allowance(AllowancesStorage::as_ref(), owner.into(), spender.into())
    }

    pub fn balance_of(&self, account: sails_rtl::ActorId) -> U256 {
        funcs::balance_of(BalancesStorage::as_ref(), account.into())
    }

    pub fn decimals(&self) -> u8 {
        MetaStorage::decimals()
    }

    // TODO (sails): allow using references.
    pub fn name(&self) -> String {
        MetaStorage::name()
    }

    pub fn symbol(&self) -> String {
        MetaStorage::symbol()
    }

    pub fn total_supply(&self) -> U256 {
        TotalSupplyStorage::get()
    }
}
