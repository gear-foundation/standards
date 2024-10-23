#![no_std]
#![allow(clippy::new_without_default)]
use crate::utils::*;
use core::fmt::Debug;
use sails_rs::{
    collections::{HashMap, HashSet},
    gstd::{msg, service},
    prelude::*,
};

pub mod funcs;
pub mod utils;

static mut STORAGE: Option<Storage> = None;

#[derive(Debug, Default)]
pub struct Storage {
    balances: HashMap<TokenId, HashMap<ActorId, U256>>,
    allowances: HashMap<ActorId, HashSet<ActorId>>,
    meta: Metadata,
    total_supply: HashMap<TokenId, U256>,
}

impl Storage {
    pub fn get_mut() -> &'static mut Self {
        unsafe { STORAGE.as_mut().expect("Storage is not initialized") }
    }
    pub fn get() -> &'static Self {
        unsafe { STORAGE.as_ref().expect("Storage is not initialized") }
    }
    pub fn balances() -> &'static mut HashMap<TokenId, HashMap<ActorId, U256>> {
        let storage = unsafe { STORAGE.as_mut().expect("Storage is not initialized") };
        &mut storage.balances
    }
    pub fn allowances() -> &'static mut HashMap<ActorId, HashSet<ActorId>> {
        let storage = unsafe { STORAGE.as_mut().expect("Storage is not initialized") };
        &mut storage.allowances
    }
    pub fn total_supply() -> &'static mut HashMap<TokenId, U256> {
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
pub enum Event {
    Approval {
        from: ActorId,
        to: ActorId,
    },
    Transfer {
        from: ActorId,
        to: ActorId,
        ids: Vec<TokenId>,
        amounts: Vec<U256>,
    },
}

#[derive(Clone)]
pub struct Service();

impl Service {
    pub fn seed(name: String, symbol: String, decimals: u8) -> Self {
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
        Self()
    }
}

#[service(events = Event)]
impl Service {
    pub fn new() -> Self {
        Self()
    }

    /// Approves an `ActorId` (account) to transfer tokens on behalf of the owner (sender).
    /// If the approval is successful, it emits an `Approval` event.
    pub fn approve(&mut self, to: ActorId) -> bool {
        let owner = msg::source();
        let storage = Storage::get_mut();
        let mutated = utils::panicking(move || funcs::approve(&mut storage.allowances, owner, to));
        if mutated {
            self.notify_on(Event::Approval { from: owner, to })
                .expect("Notification Error");
        }

        mutated
    }

    /// Transfers tokens from one account (`from`) to another (`to`) if the sender is allowed.
    /// Emits a `Transfer` event after a successful transfer.
    pub fn transfer_from(&mut self, from: ActorId, to: ActorId, id: TokenId, amount: U256) {
        let msg_src = msg::source();
        let storage = Storage::get_mut();
        let event = utils::panicking(move || {
            funcs::transfer_from(
                &mut storage.balances,
                &storage.allowances,
                msg_src,
                from,
                to,
                vec![id],
                vec![amount],
            )
        });

        self.notify_on(event).expect("Notification Error");
    }

    /// Transfers multiple tokens in batch from one account (`from`) to another (`to`).
    /// This method transfers multiple token IDs and amounts simultaneously.
    pub fn batch_transfer_from(
        &mut self,
        from: ActorId,
        to: ActorId,
        ids: Vec<TokenId>,
        amounts: Vec<U256>,
    ) {
        let msg_src = msg::source();
        let storage = Storage::get_mut();
        let event = utils::panicking(move || {
            funcs::transfer_from(
                &mut storage.balances,
                &storage.allowances,
                msg_src,
                from,
                to,
                ids,
                amounts,
            )
        });

        self.notify_on(event).expect("Notification Error");
    }

    /// Checks if a specific operator (`operator`) is approved to transfer tokens on behalf of `account`.
    /// Returns true if the operator is approved.
    pub fn is_approved(&self, account: ActorId, operator: ActorId) -> bool {
        let storage = Storage::get();
        funcs::is_approved(&storage.allowances, &account, &operator)
    }

    /// Returns the token balance of an account (`account`) for a specific token ID (`id`).
    pub fn balance_of(&self, account: ActorId, id: TokenId) -> U256 {
        let storage = Storage::get();
        funcs::get_balance(&storage.balances, &account, &id)
    }

    /// Returns token account balances (`accounts`) for specific token identifiers (`ids`).
    pub fn balance_of_batch(&self, accounts: Vec<ActorId>, ids: Vec<TokenId>) -> Vec<U256> {
        let storage = Storage::get();

        assert_eq!(
            accounts.len(),
            ids.len(),
            "Accounts and IDs must have the same length"
        );

        accounts
            .into_iter()
            .zip(ids)
            .map(|(account, id)| funcs::get_balance(&storage.balances, &account, &id))
            .collect()
    }

    /// Returns the number of decimal places used for this token.
    pub fn decimals(&self) -> &'static u8 {
        let storage = Storage::get();
        &storage.meta.decimals
    }

    /// Returns the name of the token.
    pub fn name(&self) -> &'static str {
        let storage = Storage::get();
        &storage.meta.name
    }

    /// Returns the symbol of the token.
    pub fn symbol(&self) -> &'static str {
        let storage = Storage::get();
        &storage.meta.symbol
    }

    /// Returns the total supply of tokens in circulation.
    pub fn total_supply(&self) -> Vec<(TokenId, U256)> {
        let storage = Storage::get();
        storage.total_supply.clone().into_iter().collect()
    }
}
