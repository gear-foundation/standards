use core::fmt::Debug;
use gstd::{collections::HashMap, ActorId, Decode, Encode, TypeInfo};
use sails_rtl::prelude::*;
pub type AllowancesMap = HashMap<(ActorId, ActorId), NonZeroU256>;
pub type BalancesMap = HashMap<ActorId, NonZeroU256>;
pub(crate) type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
pub enum Error {
    InsufficientAllowance,
    InsufficientBalance,
    NumericOverflow,
    Underflow,
}

