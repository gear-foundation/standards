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

    // Keeping storage compact: zero => no entry.
    if new.is_zero() {
        balances.remove(&who);
        return Ok(());
    }

    // Validate that the ABI value fits into internal compact representation and is non-zero
    let new_nz = nz_balance_from_u256(new)?;

    if balances.get(&who).is_none() {
        balances.has_space_err().map_err(map_err)?;
    }

    balances.try_insert(who, new_nz).map_err(map_err)?;
    Ok(())
}

/// Add `delta` to balance and return the new balance (ABI `U256`)
pub fn balances_add(balances: &mut BalancesMap, who: ActorId, delta: U256) -> Result<U256> {
    // No-op: keep predictable behaviour and avoid touching state
    if delta.is_zero() {
        return balances_get(balances, who);
    }

    let who_nz = nz_actor(who)?;

    let old = balances
        .get(&who_nz)
        .map(|(_, v)| u256_from_nz_balance(v))
        .unwrap_or(U256::zero());

    let new = old.checked_add(delta).ok_or(Error::NumericOverflow)?;

    if old.is_zero() && balances.get(&who_nz).is_none() {
        balances.has_space_err().map_err(map_err)?;
    }

    let new_nz = nz_balance_from_u256(new)?;

    balances.try_insert(who_nz, new_nz).map_err(map_err)?;
    Ok(new)
}

/// Subtract `delta` from balance and return the new balance (ABI `U256`)
pub fn balances_sub(balances: &mut BalancesMap, who: ActorId, delta: U256) -> Result<U256> {
    if delta.is_zero() {
        return balances_get(balances, who);
    }

    let who_nz = nz_actor(who)?;

    let old = balances
        .get(&who_nz)
        .map(|(_, v)| u256_from_nz_balance(v))
        .unwrap_or(U256::zero());

    let new = old.checked_sub(delta).ok_or(Error::InsufficientBalance)?;

    // Resulting zero => remove entry (keeps storage compact and invariant "no zero values stored").
    if new.is_zero() {
        balances.remove(&who_nz);
        return Ok(U256::zero());
    }

    let new_nz = nz_balance_from_u256(new)?;
    balances.try_insert(who_nz, new_nz).map_err(map_err)?;
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

    if allowances.get(&key).is_none() {
        allowances.has_space_err().map_err(map_err)?;
    }

    allowances.try_insert(key, new_nz).map_err(map_err)?;
    Ok(())
}
