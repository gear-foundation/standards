#![no_std]
#![allow(clippy::new_without_default)]
#![allow(unused_imports)]
#![allow(static_mut_refs)]

use core::fmt::Debug;
use sails_rs::{
    gstd::{msg, service},
    prelude::*,
};

pub mod funcs;
pub mod utils;

use crate::utils::panicking;
use utils::storage::{AllowancesMap, BalancesMap};

static mut STORAGE: Option<Storage> = None;

/// Shard sizes (valid capacities for `ShardedMap`)
/// These are "usable capacities" (7 * 2^n style) to keep allocations predictable
pub const SHARD_SMALL: usize = 7 << 14; // 114_688
pub const SHARD_MED: usize = 7 << 15; // 229_376
pub const SHARD_LARGE: usize = 7 << 16; // 458_752

/// Capacity plan for balances and allowances
/// Multiple shards allow step-wise memory growth via `alloc_next_*_shard()`
const BALANCES_CAPS: &[usize] = &[SHARD_SMALL, SHARD_MED, SHARD_MED, SHARD_LARGE];
const ALLOWANCES_CAPS: &[usize] = &[SHARD_SMALL, SHARD_SMALL, SHARD_MED, SHARD_MED];

/// Program storage: token state + metadata.
pub struct Storage {
    /// Balances map: compact, sharded, non-zero keys/values (see utils::storage)
    balances: BalancesMap,
    /// Allowances map: compact, sharded, non-zero keys/values (see utils::storage)
    allowances: AllowancesMap,
    /// Token metadata (name/symbol/decimals)
    meta: Metadata,
    /// Total minted supply
    total_supply: U256,
}

impl Storage {
    /// Mutable access to the singleton storage
    pub fn get_mut() -> &'static mut Self {
        unsafe { STORAGE.as_mut().expect("Storage is not initialized") }
    }
    /// Shared access to the singleton storage
    pub fn get() -> &'static Self {
        unsafe { STORAGE.as_ref().expect("Storage is not initialized") }
    }

    pub fn balances() -> &'static mut BalancesMap {
        &mut Self::get_mut().balances
    }

    pub fn allowances() -> &'static mut AllowancesMap {
        &mut Self::get_mut().allowances
    }

    pub fn total_supply() -> &'static mut U256 {
        &mut Self::get_mut().total_supply
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

    pub fn init(
        name: String,
        symbol: String,
        decimals: u8,
        balances_caps: Option<Vec<u32>>,
        allowances_caps: Option<Vec<u32>>,
    ) -> Self {
        // Create sharded maps with configured capacity
        let mut balances = if let Some(caps) = balances_caps {
            BalancesMap::try_new(caps.into_iter().map(|x| x as usize).collect())
                .expect("invalid balances capacities")
        } else {
            BalancesMap::try_new(BALANCES_CAPS.to_vec()).expect("invalid balances capacities")
        };

        let mut allowances = if let Some(caps) = allowances_caps {
            AllowancesMap::try_new(caps.into_iter().map(|x| x as usize).collect())
                .expect("invalid allowances capacities")
        } else {
            AllowancesMap::try_new(ALLOWANCES_CAPS.to_vec()).expect("invalid allowances capacities")
        };

        // Allocate first shard for each map
        // Further growth is explicit via `alloc_next_*_shard()` methods
        balances.alloc_next_shard();
        allowances.alloc_next_shard();

        unsafe {
            STORAGE = Some(Storage {
                balances,
                allowances,
                meta: Metadata {
                    name,
                    symbol,
                    decimals,
                },
                total_supply: U256::zero(),
            });
        }

        Self
    }
}

#[service(events = Event)]
impl Service {
    /// Allocates the next shard for balances map
    #[export]
    pub fn alloc_next_balances_shard(&mut self) -> bool {
        Storage::balances().alloc_next_shard()
    }

    /// Allocates the next shard for allowances map
    #[export]
    pub fn alloc_next_allowances_shard(&mut self) -> bool {
        Storage::allowances().alloc_next_shard()
    }

    /// Sets allowance for `spender` from msg::source() (owner)
    #[export]
    pub fn approve(&mut self, spender: ActorId, value: U256) -> bool {
        let owner = msg::source();
        let storage = Storage::get_mut();

        let mutated = panicking(|| funcs::approve(&mut storage.allowances, owner, spender, value));

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

    /// Transfers `value` from msg::source() to `to`
    #[export]
    pub fn transfer(&mut self, to: ActorId, value: U256) -> bool {
        let from = msg::source();
        let storage = Storage::get_mut();

        let mutated = panicking(move || funcs::transfer(&mut storage.balances, from, to, value));

        if mutated {
            self.emit_event(Event::Transfer { from, to, value })
                .expect("Notification Error");
        }

        mutated
    }

    /// Transfers `value` from `from` to `to` on behalf of msg::source() (spender)
    #[export]
    pub fn transfer_from(&mut self, from: ActorId, to: ActorId, value: U256) -> bool {
        let spender = msg::source();
        let storage = Storage::get_mut();

        let mutated = panicking(move || {
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

    /// Returns current allowance
    #[export]
    pub fn allowance(&self, owner: ActorId, spender: ActorId) -> U256 {
        let storage = Storage::get();
        funcs::allowance(&storage.allowances, owner, spender).unwrap_or(U256::zero())
    }

    /// Returns current balance
    #[export]
    pub fn balance_of(&self, account: ActorId) -> U256 {
        let storage = Storage::get();
        funcs::balance_of(&storage.balances, account).unwrap_or(U256::zero())
    }

    /// Token decimals
    #[export]
    pub fn decimals(&self) -> &'static u8 {
        &Storage::get().meta.decimals
    }

    /// Token name
    #[export]
    pub fn name(&self) -> &'static str {
        &Storage::get().meta.name
    }

    /// Token symbol
    #[export]
    pub fn symbol(&self) -> &'static str {
        &Storage::get().meta.symbol
    }

    /// Total supply
    #[export]
    pub fn total_supply(&self) -> &'static U256 {
        &Storage::get().total_supply
    }

    /// Balances map stats: (len, allocated_capacity, max_capacity, free_space)
    #[export]
    pub fn balances_stats(&self) -> (u128, u128, u128, u128) {
        let b = &Storage::get().balances;
        (
            b.len() as u128,
            b.capacity() as u128,
            b.max_capacity() as u128,
            b.space() as u128,
        )
    }

    /// Allowances map stats: (len, allocated_capacity, max_capacity, free_space)
    #[export]
    pub fn allowances_stats(&self) -> (u128, u128, u128, u128) {
        let a = &Storage::get().allowances;
        (
            a.len() as u128,
            a.capacity() as u128,
            a.max_capacity() as u128,
            a.space() as u128,
        )
    }
}
