#![allow(static_mut_refs)]

use sails_rs::{
    collections::HashSet,
    gstd::{msg, service, services::Service},
    prelude::*,
};

mod funcs;
use crate::services;
use vft_service::{Service as VftService, ServiceExposure, Storage};

/// Extra state for the extended token: role-based access control
/// Stored separately from the base VFT storage
#[derive(Default)]
pub struct ExtendedStorage {
    /// Accounts allowed to mint.
    minters: HashSet<ActorId>,
    /// Accounts allowed to burn.
    burners: HashSet<ActorId>,
    /// Accounts allowed to grant/revoke roles and grow storage.
    admins: HashSet<ActorId>,
}

static mut EXTENDED_STORAGE: Option<ExtendedStorage> = None;

#[event]
#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum Event {
    Minted { to: ActorId, value: U256 },
    Burned { from: ActorId, value: U256 },
}

/// Extended service: composes the base VFT service and adds roles + mint/burn
#[derive(Clone)]
pub struct ExtendedService {
    vft: VftService,
}

impl ExtendedService {
    pub fn new() -> Self {
        Self {
            vft: VftService::new(),
        }
    }

    /// Initialization:
    /// - sets msg::source() as initial admin/minter/burner
    /// - initializes the base VFT storage (name/symbol/decimals)
    pub fn init(
        name: String,
        symbol: String,
        decimals: u8,
        balances_caps: Option<Vec<u32>>,
        allowances_caps: Option<Vec<u32>>,
    ) -> Self {
        let admin = msg::source();

        unsafe {
            EXTENDED_STORAGE = Some(ExtendedStorage {
                admins: [admin].into(),
                minters: [admin].into(),
                burners: [admin].into(),
            });
        }

        ExtendedService {
            vft: VftService::init(name, symbol, decimals, balances_caps, allowances_caps),
        }
    }

    /// Mutable access to extension storage
    pub fn get_mut(&mut self) -> &'static mut ExtendedStorage {
        unsafe {
            EXTENDED_STORAGE
                .as_mut()
                .expect("Extended vft is not initialized")
        }
    }

    /// Shared access to extension storage
    pub fn get(&self) -> &'static ExtendedStorage {
        unsafe {
            EXTENDED_STORAGE
                .as_ref()
                .expect("Extended vft is not initialized")
        }
    }
}

/// Allows treating `ExtendedService` as `VftService` when needed
impl From<ExtendedService> for VftService {
    fn from(value: ExtendedService) -> Self {
        value.vft
    }
}

/// `extends = VftService` exposes base VFT exports on this service
#[service(extends = VftService, events = Event)]
impl ExtendedService {
    fn vft(&self) -> ServiceExposure<VftService> {
        VftService::new().expose(self.route())
    }

    /// Append a new shard to balances with the given capacity
    #[export]
    pub fn append_balances_shard(&mut self, cap: u32) {
        self.ensure_is_admin();
        self.vft().append_balances_shard(cap);
    }
    /// Append a new shard to allowances with the given capacity
    #[export]
    pub fn append_allowances_shard(&mut self, cap: u32) {
        self.ensure_is_admin();
        self.vft().append_allowances_shard(cap);
    }

    /// Explicitly grow balances map
    #[export]
    pub fn alloc_next_balances_shard(&mut self) -> bool {
        self.ensure_is_admin();
        self.vft().alloc_next_balances_shard()
    }

    /// Explicitly grow allowances map
    #[export]
    pub fn alloc_next_allowances_shard(&mut self) -> bool {
        self.ensure_is_admin();
        self.vft().alloc_next_balances_shard()
    }

    /// Mint new tokens
    #[export]
    pub fn mint(&mut self, to: ActorId, value: U256) -> bool {
        // Authorization check (role-based access control).
        if !self.get().minters.contains(&msg::source()) {
            panic!("Not allowed to mint");
        }

        // Execute mint logic transactionally (panic on error).
        let mutated = services::utils::panicking(|| {
            funcs::mint(Storage::balances(), Storage::total_supply(), to, value)
        });

        // Emit extension event only when state actually changed.
        if mutated {
            self.emit_event(Event::Minted { to, value })
                .expect("Notification Error");
        }

        mutated
    }

    /// Burn tokens
    #[export]
    pub fn burn(&mut self, from: ActorId, value: U256) -> bool {
        if !self.get().burners.contains(&msg::source()) {
            panic!("Not allowed to burn");
        }

        let mutated = services::utils::panicking(|| {
            funcs::burn(Storage::balances(), Storage::total_supply(), from, value)
        });

        if mutated {
            self.emit_event(Event::Burned { from, value })
                .expect("Notification Error");
        }

        mutated
    }

    /// Grant admin role
    #[export]
    pub fn grant_admin_role(&mut self, to: ActorId) {
        self.ensure_is_admin();
        self.get_mut().admins.insert(to);
    }

    /// Grant minter role
    #[export]
    pub fn grant_minter_role(&mut self, to: ActorId) {
        self.ensure_is_admin();
        self.get_mut().minters.insert(to);
    }

    /// Grant burner role
    #[export]
    pub fn grant_burner_role(&mut self, to: ActorId) {
        self.ensure_is_admin();
        self.get_mut().burners.insert(to);
    }

    /// Revoke admin role
    #[export]
    pub fn revoke_admin_role(&mut self, from: ActorId) {
        self.ensure_is_admin();
        self.get_mut().admins.remove(&from);
    }

    /// Revoke minter role
    #[export]
    pub fn revoke_minter_role(&mut self, from: ActorId) {
        self.ensure_is_admin();
        self.get_mut().minters.remove(&from);
    }

    /// Revoke burner role
    #[export]
    pub fn revoke_burner_role(&mut self, from: ActorId) {
        self.ensure_is_admin();
        self.get_mut().burners.remove(&from);
    }

    /// Returns current minters list (unordered)
    #[export]
    pub fn minters(&self) -> Vec<ActorId> {
        self.get().minters.clone().into_iter().collect()
    }

    /// Returns current burners list (unordered)
    #[export]
    pub fn burners(&self) -> Vec<ActorId> {
        self.get().burners.clone().into_iter().collect()
    }

    /// Returns current admins list (unordered)
    #[export]
    pub fn admins(&self) -> Vec<ActorId> {
        self.get().admins.clone().into_iter().collect()
    }

    /// This method only works with the feature `stress-tests`.
    /// It is necessary for quickly filling in the state in the test.
    #[export]
    pub fn mint_range(&mut self, start: u64, count: u32, value: U256) -> u32 {
        #[cfg(feature = "stress-tests")]
        {
            self.ensure_is_admin();

            if count == 0 || value.is_zero() {
                return 0;
            }

            let balances = Storage::balances();
            let total_supply = Storage::total_supply();

            let val_nz = vft_service::utils::helpers::nz_balance_from_u256(value)
                .unwrap_or_else(|_| panic!("BadValue"));
            let mut minted: u32 = 0;

            for i in 0..count {
                let user: ActorId = (start + i as u64).into();
                let who = match vft_service::utils::helpers::nz_actor(user) {
                    Ok(x) => x,
                    Err(_) => break,
                };

                // ВАЖНО: unsafe — потому что мы предполагаем, что ключа ещё нет
                let res = unsafe { balances.try_insert_new(who, val_nz) };
                if res.is_err() {
                    break; // CapacityOverflow и т.п.
                }

                minted += 1;
            }

            if minted > 0 {
                let delta = value
                    .checked_mul(U256::from(minted))
                    .unwrap_or_else(|| panic!("NumericOverflow"));
                *total_supply = total_supply
                    .checked_add(delta)
                    .unwrap_or_else(|| panic!("NumericOverflow"));
            }

            minted
        }

        #[cfg(not(feature = "stress-tests"))]
        {
            let _ = (start, count, value);
            panic!("This method only works with `stress-tests` feature");
        }
    }
}

impl ExtendedService {
    /// Admin gate for privileged operations (roles + storage growth)
    fn ensure_is_admin(&self) {
        if !self.get().admins.contains(&msg::source()) {
            panic!("Not admin");
        }
    }
}
