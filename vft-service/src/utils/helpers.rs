use core::fmt::Debug;
use gstd::ext;
use sails_rs::prelude::*;

use awesome_sails_utils::map::ShardedMap;
use awesome_sails_utils::math::{NonZero, OverflowError, U256};

use crate::utils::error::{map_err, Error};
use crate::utils::storage::{Allowance, Balance, NonZeroActorId, NzAllowance, NzBalance, Result};

#[inline]
pub fn nz_actor(id: ActorId) -> Result<NonZeroActorId> {
    NonZeroActorId::try_from(id).map_err(|_| Error::ZeroActor)
}

#[inline]
pub fn nz_balance_from_u256(value: U256) -> Result<NzBalance> {
    if value.is_zero() {
        return Err(Error::ZeroAmount);
    }
    let b = Balance::try_from(value).map_err(|_e: OverflowError| Error::AmountOverflow)?;
    NonZero::try_new(b).map_err(|_| Error::ZeroAmount)
}

#[inline]
pub fn nz_allowance_from_u256(value: U256) -> Result<NzAllowance> {
    if value.is_zero() {
        return Err(Error::ZeroAmount);
    }
    let a = Allowance::try_from(value).map_err(|_e: OverflowError| Error::AmountOverflow)?;
    NonZero::try_new(a).map_err(|_| Error::ZeroAmount)
}

#[inline]
pub fn u256_from_nz_balance(v: &NzBalance) -> U256 {
    U256::from(Balance::from(*v))
}

#[inline]
pub fn u256_from_nz_allowance(v: &NzAllowance) -> U256 {
    U256::from(Allowance::from(*v))
}

#[inline]
pub fn upsert_nz<K, V>(
    map: &mut ShardedMap<K, NonZero<V>>,
    key: K,
    value: NonZero<V>,
) -> Result<bool>
where
    K: Eq + core::hash::Hash,
    V: Copy + PartialEq,
{
    let (_idx, prev) = map.try_insert(key, value).map_err(map_err)?;
    Ok(prev.map(|p| p != value).unwrap_or(true))
}

#[inline]
pub fn remove_key<K, V>(map: &mut ShardedMap<K, V>, key: &K) -> bool
where
    K: Eq + core::hash::Hash,
{
    map.remove(key).is_some()
}

pub fn panicking<T, E: Debug, F: FnOnce() -> core::result::Result<T, E>>(f: F) -> T {
    match f() {
        Ok(v) => v,
        Err(e) => panic(e),
    }
}

pub fn panic(err: impl Debug) -> ! {
    ext::panic(format!("{err:?}"))
}
