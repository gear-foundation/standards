use sails_rs::prelude::*;
use vft_service::utils::{self, Error, Result};

/// Mints `value` tokens to `to` and increases total supply
pub fn mint(
    balances: &mut utils::BalancesMap,
    total_supply: &mut U256,
    to: ActorId,
    value: U256,
) -> Result<bool> {
    // No-op mint for zero value
    if value.is_zero() {
        return Ok(false);
    }

    let new_total = total_supply
        .checked_add(value)
        .ok_or(Error::NumericOverflow)?;

    utils::balances_add(balances, to, value)?;

    *total_supply = new_total;
    Ok(true)
}

/// Burns `value` tokens from `from` and decreases total supply
pub fn burn(
    balances: &mut utils::BalancesMap,
    total_supply: &mut U256,
    from: ActorId,
    value: U256,
) -> Result<bool> {
    // No-op burn for zero value
    if value.is_zero() {
        return Ok(false);
    }

    let new_total = total_supply.checked_sub(value).ok_or(Error::Underflow)?;

    utils::balances_sub(balances, from, value)?;

    *total_supply = new_total;
    Ok(true)
}
