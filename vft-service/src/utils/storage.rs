use core::fmt::Debug;
use sails_rs::prelude::*;

use awesome_sails_utils::impl_math_wrapper;
use awesome_sails_utils::map::{ShardedMap, ShardedMapError};
use awesome_sails_utils::math::OverflowError;
use awesome_sails_utils::math::{LeBytes, NonZero};

use crate::impl_math_wrapper_any;
use crate::utils::error::Error;

#[cfg(feature = "amount_80")]
pub const BALANCE_BYTES: usize = 10;
#[cfg(feature = "amount_96")]
pub const BALANCE_BYTES: usize = 12;
#[cfg(feature = "amount_128")]
pub const BALANCE_BYTES: usize = 16;
#[cfg(feature = "amount_160")]
pub const BALANCE_BYTES: usize = 20;
#[cfg(feature = "amount_192")]
pub const BALANCE_BYTES: usize = 24;
#[cfg(feature = "amount_224")]
pub const BALANCE_BYTES: usize = 28;
#[cfg(feature = "amount_256")]
pub const BALANCE_BYTES: usize = 32;

pub const ALLOWANCE_BYTES: usize = BALANCE_BYTES;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct Balance(pub LeBytes<BALANCE_BYTES>);

impl_math_wrapper_any!(Balance, LeBytes<BALANCE_BYTES>);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct Allowance(LeBytes<ALLOWANCE_BYTES>);

impl_math_wrapper_any!(Allowance, LeBytes<ALLOWANCE_BYTES>);

// ---------------------------
// Storage aliases
// ---------------------------

pub type NonZeroActorId = NonZero<ActorId>;
pub type NzBalance = NonZero<Balance>;
pub type NzAllowance = NonZero<Allowance>;

pub type BalancesMap = ShardedMap<NonZeroActorId, NzBalance>;
pub type AllowancesMap = ShardedMap<(NonZeroActorId, NonZeroActorId), NzAllowance>;
pub type Result<T, E = Error> = core::result::Result<T, E>;
