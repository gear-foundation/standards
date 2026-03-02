pub mod error;
pub mod facade;
pub mod helpers;
pub mod macros;
pub mod storage;

pub use error::{map_err, Error};
pub use storage::{
    Allowance, AllowancesMap, Balance, BalancesMap, NonZeroActorId, NzAllowance, NzBalance, Result,
};

// facade API (what extended-vft should use)
pub use facade::{
    allowances_get, allowances_set, balances_add, balances_get, balances_set, balances_sub,
};

// keep these available for Service
pub use helpers::{panic, panicking};
