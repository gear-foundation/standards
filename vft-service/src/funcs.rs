use sails_rs::prelude::U256;
use sails_rs::ActorId;

use crate::utils::helpers::{
    nz_actor, nz_allowance_from_u256, nz_balance_from_u256, remove_key, u256_from_nz_allowance,
    u256_from_nz_balance, upsert_nz,
};
use crate::utils::storage::{AllowancesMap, BalancesMap, NonZeroActorId, Result};
use crate::utils::{map_err, Error};

fn transfer_nz(
    balances: &mut BalancesMap,
    from: NonZeroActorId,
    to: NonZeroActorId,
    value: U256,
) -> Result<bool> {
    // No-op on self transfer
    if from == to {
        return Ok(false);
    }

    // Strict: zero amount is invalid for transfers
    if value.is_zero() {
        return Err(Error::ZeroAmount);
    }

    // Ensure `value` fits compact Balance (LeBytes) and can be stored as NonZero
    let _ = nz_balance_from_u256(value)?;

    let from_u256 = balances
        .get(&from)
        .map(|(_, v)| u256_from_nz_balance(v))
        .unwrap_or(U256::zero());

    let new_from_u256 = from_u256
        .checked_sub(value)
        .ok_or(Error::InsufficientBalance)?;

    let to_opt = balances.get(&to).map(|(_, v)| u256_from_nz_balance(v));
    let to_exists = to_opt.is_some();
    let to_u256 = to_opt.unwrap_or(U256::zero());

    let new_to_u256 = to_u256.checked_add(value).ok_or(Error::NumericOverflow)?;

    if !to_exists && !new_from_u256.is_zero() {
        balances.has_space_err().map_err(map_err)?;
    }

    if !new_from_u256.is_zero() {
        let new_from = nz_balance_from_u256(new_from_u256)?;
        balances.try_insert(from, new_from).map_err(map_err)?;
    } else {
        balances.remove(&from);
    }
    let new_to = nz_balance_from_u256(new_to_u256)?;
    balances.try_insert(to, new_to).map_err(map_err)?;

    Ok(true)
}

pub fn allowance(allowances: &AllowancesMap, owner: ActorId, spender: ActorId) -> Result<U256> {
    let owner = nz_actor(owner)?;
    let spender = nz_actor(spender)?;

    Ok(allowances
        .get(&(owner, spender))
        .map(|(_, v)| u256_from_nz_allowance(v))
        .unwrap_or(U256::zero()))
}

pub fn balance_of(balances: &BalancesMap, owner: ActorId) -> Result<U256> {
    let owner = nz_actor(owner)?;

    Ok(balances
        .get(&owner)
        .map(|(_, v)| u256_from_nz_balance(v))
        .unwrap_or(U256::zero()))
}

pub fn approve(
    allowances: &mut AllowancesMap,
    owner: ActorId,
    spender: ActorId,
    value: U256,
) -> Result<bool> {
    let owner = nz_actor(owner)?;
    let spender = nz_actor(spender)?;

    if owner == spender {
        return Ok(false);
    }

    let key = (owner, spender);

    // Setting to zero removes the entry (keeps state compact)
    if value.is_zero() {
        return Ok(remove_key(allowances, &key));
    }

    // Validate `value` fits compact Allowance and is non-zero
    let v = nz_allowance_from_u256(value)?;

    // New key => may require capacity
    if allowances.get(&key).is_none() {
        allowances.has_space_err().map_err(map_err)?;
    }

    // Upsert and report whether state changed
    upsert_nz(allowances, key, v)
}

pub fn transfer(
    balances: &mut BalancesMap,
    from: ActorId,
    to: ActorId,
    value: U256,
) -> Result<bool> {
    let from = nz_actor(from)?;
    let to = nz_actor(to)?;
    transfer_nz(balances, from, to, value)
}

pub fn transfer_from(
    allowances: &mut AllowancesMap,
    balances: &mut BalancesMap,
    spender: ActorId,
    from: ActorId,
    to: ActorId,
    value: U256,
) -> Result<bool> {
    // Strict: zero amount is invalid.
    if value.is_zero() {
        return Err(Error::ZeroAmount);
    }

    let spender = nz_actor(spender)?;
    let from_nz = nz_actor(from)?;
    let to_nz = nz_actor(to)?;

    // Shortcut: spender == owner, no allowance required.
    if spender == from_nz {
        return transfer_nz(balances, from_nz, to_nz, value);
    }

    // No-op on self transfer.
    if from_nz == to_nz {
        return Ok(false);
    }

    // Validate amount fits compact Allowance and is non-zero.
    let _ = nz_allowance_from_u256(value)?;

    let key = (from_nz, spender);

    // Missing entry => allowance is 0.
    let cur_allow_u256 = allowances
        .get(&key)
        .map(|(_, v)| u256_from_nz_allowance(v))
        .unwrap_or(U256::zero());

    // Underflow => insufficient allowance.
    let new_allow_u256 = cur_allow_u256
        .checked_sub(value)
        .ok_or(Error::InsufficientAllowance)?;

    // Transactional transfer (will precheck capacity internally if needed).
    let did_transfer = transfer_nz(balances, from_nz, to_nz, value)?;
    debug_assert!(did_transfer);

    // Update/remove allowance entry.
    if !new_allow_u256.is_zero() {
        let new_allow = nz_allowance_from_u256(new_allow_u256)?;
        allowances.try_insert(key, new_allow).map_err(map_err)?;
    } else {
        allowances.remove(&key);
    }

    Ok(true)
}
