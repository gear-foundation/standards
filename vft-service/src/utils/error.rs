use awesome_sails_utils::map::ShardedMapError;
use sails_rs::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
pub enum Error {
    InsufficientAllowance,
    InsufficientBalance,
    NumericOverflow,
    Underflow,
    CapacityOverflow,
    ZeroAmount,
    ZeroActor,
    AmountOverflow,
}

#[inline]
pub fn map_err(e: ShardedMapError) -> Error {
    match e {
        ShardedMapError::CapacityOverflow => Error::CapacityOverflow,
        ShardedMapError::InvalidCapacity => unreachable!("capacity is validated at init"),
    }
}
