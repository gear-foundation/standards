use core::fmt::Debug;
use gstd::ext;
use sails_rs::prelude::*;
use sails_rs::{collections::HashMap, prelude::*};
pub type AllowancesMap = HashMap<(ActorId, ActorId), U256>;
pub type BalancesMap = HashMap<ActorId, U256>;
pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
pub enum Error {
    InsufficientAllowance,
    InsufficientBalance,
    NumericOverflow,
    Underflow,
}

pub fn panicking<T, E: Debug, F: FnOnce() -> Result<T, E>>(f: F) -> T {
    match f() {
        Ok(v) => v,
        Err(e) => panic(e),
    }
}

pub fn panic(err: impl Debug) -> ! {
    ext::panic(format!("{err:?}"))
}
