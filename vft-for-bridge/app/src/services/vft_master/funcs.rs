use super::vft::{
    funcs,
    utils::{Result, *},
};
use gstd::{prelude::*, ActorId};
use primitive_types::U256;

pub fn batch_mint(
    balances: &mut BalancesMap,
    total_supply: &mut U256,
    to: Vec<ActorId>,
    value: Vec<U256>,
) -> Result<bool> {
    // Ensure the lists of addresses and values are of equal length
    if to.len() != value.len() {
        return Err(Error::InvalidInput); 
    }

    // Iterate over pairs (actor, value)
    for (actor, val) in to.into_iter().zip(value.into_iter()) {
        // Call the mint function for each pair
        mint(balances, total_supply, actor, val)?;
    }

    Ok(true)
}

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

    let Ok(non_zero_new_to) = new_to.try_into() else {
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

    if let Ok(non_zero_new_from) = new_from.try_into() {
        balances.insert(from, non_zero_new_from);
    } else {
        balances.remove(&from);
    }
    *total_supply = new_total_supply;
    Ok(true)
}
