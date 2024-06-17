use core::fmt::Debug;
use gstd::{collections::HashMap, format, msg, ActorId, Decode, Encode, String, TypeInfo, Vec};
use primitive_types::U256;
use sails_rtl::gstd::gservice;

pub use utils::*;

pub mod funcs;
pub(crate) mod utils;

static mut PROGRAM: Option<Program> = None;

#[derive(Debug, Default)]
pub struct Program {
    pub balances: HashMap<ActorId, NonZeroU256>,
    pub allowances: HashMap<(ActorId, ActorId), NonZeroU256>,
    pub meta: Metadata,
    pub total_supply: U256,
}

impl Program {
    pub fn get_mut() -> &'static mut Self {
        unsafe { PROGRAM.as_mut().expect("Program is not initialized") }
    }
    pub fn get() -> &'static Self {
        unsafe { PROGRAM.as_ref().expect("Program is not initialized") }
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

pub type GstdDrivenService = Service;

#[derive(Clone)]
pub struct Service();

impl Service {
    pub fn seed(name: String, symbol: String, decimals: u8) -> Self {
        unsafe {
            PROGRAM = Some(Program {
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
        let program = Program::get_mut();
        let mutated = funcs::approve(&mut program.allowances, owner, spender.into(), value);

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
        let program = Program::get_mut();
        let mutated =
            panicking(move || funcs::transfer(&mut program.balances, from, to.into(), value));

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
        let program = Program::get_mut();
        let mutated = panicking(move || {
            funcs::transfer_from(
                &mut program.allowances,
                &mut program.balances,
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
        let program = Program::get();
        funcs::allowance(&program.allowances, owner.into(), spender.into())
    }

    pub fn balance_of(&self, account: sails_rtl::ActorId) -> U256 {
        let program = Program::get();
        funcs::balance_of(&program.balances, account.into())
    }

    pub fn decimals(&self) -> u8 {
        let program = Program::get();
        program.meta.decimals
    }

    pub fn name(&self) -> String {
        let program = Program::get();
        program.meta.name.clone()
    }

    pub fn symbol(&self) -> String {
        let program = Program::get();
        program.meta.symbol.clone()
    }

    pub fn total_supply(&self) -> U256 {
        let program = Program::get();
        program.total_supply
    }
}
