//! Facade over internal storage types (`ShardedMap<NonZero<...>>`) that exposes
//! an ABI-friendly interface: `ActorId` + `U256`.
//!
//! Why this exists:
//! - Extended implementations should not depend on `NonZero`, `LeBytes<N>`, shard indexes, etc.
//! - All state-changing operations remain transactional: we either commit fully or return an error
//!   before touching state (except for explicit shard allocation entrypoints elsewhere).

use sails_rs::prelude::*;

use crate::utils::helpers::{
    nz_actor, nz_allowance_from_u256, nz_balance_from_u256, u256_from_nz_allowance,
    u256_from_nz_balance,
};

use crate::utils::error::{map_err, Error};
use crate::utils::storage::{AllowancesMap, BalancesMap, Result};

/// Read balance in ABI form (`U256`).
///
/// Strictness:
/// - `ActorId == 0` is rejected with `Error::ZeroActor` (we never store zero ActorId keys)
pub fn balances_get(balances: &BalancesMap, who: ActorId) -> Result<U256> {
    // Convert to NonZeroActorId (or error)
    let who = nz_actor(who)?;

    Ok(balances
        .get(&who)
        .map(|(_, v)| u256_from_nz_balance(v))
        .unwrap_or(U256::zero()))
}

/// Set balance to `new` (ABI `U256`).
///
/// Storage policy:
/// - `new == 0` removes the entry (we do not store zero balances).
pub fn balances_set(balances: &mut BalancesMap, who: ActorId, new: U256) -> Result<()> {
    let who = nz_actor(who)?;

    if new.is_zero() {
        balances.remove(&who);
        return Ok(());
    }

    let new_nz = nz_balance_from_u256(new)?;

    if let Some((_idx, v_mut)) = balances.get_mut(&who) {
        *v_mut = new_nz;
        return Ok(());
    }

    unsafe { balances.try_insert_new(who, new_nz) }.map_err(map_err)?;
    Ok(())
}

/// Add `delta` to balance and return the new balance (ABI `U256`)
pub fn balances_add(balances: &mut BalancesMap, who: ActorId, delta: U256) -> Result<U256> {
    if delta.is_zero() {
        return balances_get(balances, who);
    }

    let who = nz_actor(who)?;

    if let Some((_idx, v_mut)) = balances.get_mut(&who) {
        let old = u256_from_nz_balance(v_mut);
        let new = old.checked_add(delta).ok_or(Error::NumericOverflow)?;
        *v_mut = nz_balance_from_u256(new)?;
        return Ok(new);
    }

    let new_nz = nz_balance_from_u256(delta)?;
    unsafe { balances.try_insert_new(who, new_nz) }.map_err(map_err)?;
    Ok(delta)
}

/// Subtract `delta` from balance and return the new balance (ABI `U256`)
pub fn balances_sub(balances: &mut BalancesMap, who: ActorId, delta: U256) -> Result<U256> {
    if delta.is_zero() {
        return balances_get(balances, who);
    }

    let who = nz_actor(who)?;

    let Some((_idx, v_mut)) = balances.get_mut(&who) else {
        return Err(Error::InsufficientBalance);
    };

    let old = u256_from_nz_balance(v_mut);
    let new = old.checked_sub(delta).ok_or(Error::InsufficientBalance)?;

    if new.is_zero() {
        balances.remove(&who);
        return Ok(U256::zero());
    }

    *v_mut = nz_balance_from_u256(new)?;
    Ok(new)
}

/// Read allowance in ABI form (`U256`)
pub fn allowances_get(
    allowances: &AllowancesMap,
    owner: ActorId,
    spender: ActorId,
) -> Result<U256> {
    let owner = nz_actor(owner)?;
    let spender = nz_actor(spender)?;

    Ok(allowances
        .get(&(owner, spender))
        .map(|(_, v)| u256_from_nz_allowance(v))
        .unwrap_or(U256::zero()))
}

/// Set allowance to `new` (ABI `U256`)
pub fn allowances_set(
    allowances: &mut AllowancesMap,
    owner: ActorId,
    spender: ActorId,
    new: U256,
) -> Result<()> {
    let owner = nz_actor(owner)?;
    let spender = nz_actor(spender)?;
    let key = (owner, spender);

    if new.is_zero() {
        allowances.remove(&key);
        return Ok(());
    }

    let new_nz = nz_allowance_from_u256(new)?;

    if let Some((_idx, v_mut)) = allowances.get_mut(&key) {
        *v_mut = new_nz;
        return Ok(());
    }

    unsafe { allowances.try_insert_new(key, new_nz) }.map_err(map_err)?;
    Ok(())
}
