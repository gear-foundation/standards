use core::fmt::Debug;
use gstd::{collections::HashMap, ActorId, Decode, Encode, TypeInfo};
use primitive_types::U256;

pub type AllowancesMap = HashMap<(ActorId, ActorId), NonZeroU256>;
pub type BalancesMap = HashMap<ActorId, NonZeroU256>;

pub(crate) type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
pub enum Error {
    InsufficientAllowance,
    InsufficientBalance,
    NumericOverflow,
    Underflow,
    InvalidInput,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Decode, Encode, TypeInfo)]
pub struct NonZeroU256(U256);

impl TryFrom<U256> for NonZeroU256 {
    type Error = TryFromU256Error;

    fn try_from(value: U256) -> Result<Self, Self::Error> {
        (!value.is_zero())
            .then_some(NonZeroU256(value))
            .ok_or(TryFromU256Error(()))
    }
}

impl From<NonZeroU256> for U256 {
    fn from(value: NonZeroU256) -> Self {
        value.0
    }
}

#[derive(Debug)]
pub struct TryFromU256Error(());
