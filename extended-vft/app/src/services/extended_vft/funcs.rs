use gstd::{prelude::*, ActorId};
use sails::prelude::*;
use vft::{
    funcs,
    utils::{Error, Result, *},
};

pub fn mint(
    balances: &mut BalancesMap,
    total_supply: &mut U256,
    to: ActorId,
    value: U256,
) -> Result<bool> {
    if value.is_zero() {
        return Ok(false);
    }

    let new_total_supply = total_supply
        .checked_add(value)
        .ok_or(Error::NumericOverflow)?;

    let new_to = funcs::balance_of(balances, to)
        .checked_add(value)
        .ok_or(Error::NumericOverflow)?;

    let Some(non_zero_new_to) = NonZeroU256::new(new_to) else {
        unreachable!("Infallible since fn is noop on zero value; qed");
    };

    balances.insert(to, non_zero_new_to);
    *total_supply = new_total_supply;

    Ok(true)
}

pub fn burn(
    balances: &mut BalancesMap,
    total_supply: &mut U256,
    from: ActorId,
    value: U256,
) -> Result<bool> {
    if value.is_zero() {
        return Ok(false);
    }
    let new_total_supply = total_supply.checked_sub(value).ok_or(Error::Underflow)?;

    let new_from = funcs::balance_of(balances, from)
        .checked_sub(value)
        .ok_or(Error::InsufficientBalance)?;

    if let Some(non_zero_new_from) = NonZeroU256::new(new_from) {
        balances.insert(from, non_zero_new_from);
    } else {
        balances.remove(&from);
    }
    *total_supply = new_total_supply;
    Ok(true)
}
