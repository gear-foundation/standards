use core::fmt::Debug;
use sails_rs::prelude::*;

use awesome_sails_utils::impl_math_wrapper;
use awesome_sails_utils::map::{ShardedMap, ShardedMapError};
use awesome_sails_utils::math::OverflowError;
use awesome_sails_utils::math::{LeBytes, NonZero};

use crate::utils::error::Error;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct Balance(pub LeBytes<10>);

impl_math_wrapper!(Balance, LeBytes<10>);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct Allowance(LeBytes<9>);

impl_math_wrapper!(Allowance, LeBytes<9>);

impl From<u64> for Balance {
    fn from(value: u64) -> Self {
        Balance::try_from(u128::from(value)).expect("u64 always fits into Balance")
    }
}

// ---------------------------
// Storage aliases
// ---------------------------

pub type NonZeroActorId = NonZero<ActorId>;
pub type NzBalance = NonZero<Balance>;
pub type NzAllowance = NonZero<Allowance>;

pub type BalancesMap = ShardedMap<NonZeroActorId, NzBalance>;
pub type AllowancesMap = ShardedMap<(NonZeroActorId, NonZeroActorId), NzAllowance>;
pub type Result<T, E = Error> = core::result::Result<T, E>;
