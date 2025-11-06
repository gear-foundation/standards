#![allow(static_mut_refs)]
use sails_rs::{
    collections::{HashMap, HashSet},
    gstd::{msg, service},
    prelude::*,
};
mod funcs;
use crate::services;
pub mod utils;
use utils::*;
use vmt_service::{Service as VmtService, Storage};

#[derive(Default)]
pub struct ExtendedStorage {
    token_metadata: HashMap<TokenId, TokenMetadata>,
    owners: HashMap<TokenId, ActorId>,
    minters: HashSet<ActorId>,
    burners: HashSet<ActorId>,
    admins: HashSet<ActorId>,
}

static mut EXTENDED_STORAGE: Option<ExtendedStorage> = None;

#[event]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum Event {
    Minted {
        to: ActorId,
        ids: Vec<TokenId>,
        amounts: Vec<U256>,
    },
    Burned {
        from: ActorId,
        ids: Vec<TokenId>,
        amounts: Vec<U256>,
    },
}
#[derive(Clone)]
pub struct ExtendedService {
    vmt: VmtService,
}

impl ExtendedService {
    pub fn new() -> Self {
        Self {
            vmt: VmtService::new(),
        }
    }
    pub fn init(name: String, symbol: String, decimals: u8) -> Self {
        let admin = msg::source();
        unsafe {
            EXTENDED_STORAGE = Some(ExtendedStorage {
                token_metadata: HashMap::new(),
                owners: HashMap::new(),
                admins: [admin].into(),
                minters: [admin].into(),
                burners: [admin].into(),
            });
        };
        ExtendedService {
            vmt: <VmtService>::init(name, symbol, decimals),
        }
    }

    pub fn get_mut(&mut self) -> &'static mut ExtendedStorage {
        unsafe {
            EXTENDED_STORAGE
                .as_mut()
                .expect("Extended vmt is not initialized")
        }
    }
    pub fn get(&self) -> &'static ExtendedStorage {
        unsafe {
            EXTENDED_STORAGE
                .as_ref()
                .expect("Extended vmt is not initialized")
        }
    }
}

impl From<ExtendedService> for VmtService {
    fn from(value: ExtendedService) -> Self {
        value.vmt
    }
}

#[service(extends = VmtService, events = Event)]
impl ExtendedService {
    #[export]
    pub fn mint(
        &mut self,
        to: ActorId,
        id: TokenId,
        amount: U256,
        token_metadata: Option<TokenMetadata>,
    ) {
        if !self.get().minters.contains(&msg::source()) {
            panic!("Not allowed to mint")
        };

        let event = services::utils::panicking(|| {
            funcs::mint(
                Storage::balances(),
                Storage::total_supply(),
                self.get_mut(),
                to,
                vec![id],
                vec![amount],
                vec![token_metadata],
            )
        });
        self.emit_event(event).expect("Notification Error");
    }

    #[export]
    pub fn mint_batch(
        &mut self,
        to: ActorId,
        ids: Vec<TokenId>,
        amounts: Vec<U256>,
        token_metadata: Vec<Option<TokenMetadata>>,
    ) {
        if !self.get().minters.contains(&msg::source()) {
            panic!("Not allowed to mint")
        };

        let event = services::utils::panicking(|| {
            funcs::mint(
                Storage::balances(),
                Storage::total_supply(),
                self.get_mut(),
                to,
                ids,
                amounts,
                token_metadata,
            )
        });
        self.emit_event(event).expect("Notification Error");
    }

    #[export]
    pub fn burn(&mut self, from: ActorId, id: TokenId, amount: U256) {
        if !self.get().burners.contains(&msg::source()) {
            panic!("Not allowed to burn")
        };

        let event = services::utils::panicking(|| {
            funcs::burn(
                Storage::balances(),
                Storage::total_supply(),
                self.get_mut(),
                from,
                vec![id],
                vec![amount],
            )
        });
        self.emit_event(event).expect("Notification Error");
    }

    #[export]
    pub fn burn_batch(&mut self, from: ActorId, ids: Vec<TokenId>, amounts: Vec<U256>) {
        if !self.get().burners.contains(&msg::source()) {
            panic!("Not allowed to burn")
        };

        let event = services::utils::panicking(|| {
            funcs::burn(
                Storage::balances(),
                Storage::total_supply(),
                self.get_mut(),
                from,
                ids,
                amounts,
            )
        });
        self.emit_event(event).expect("Notification Error");
    }

    #[export]
    pub fn grant_admin_role(&mut self, to: ActorId) {
        self.ensure_is_admin();
        self.get_mut().admins.insert(to);
    }

    #[export]
    pub fn grant_minter_role(&mut self, to: ActorId) {
        self.ensure_is_admin();
        self.get_mut().minters.insert(to);
    }

    #[export]
    pub fn grant_burner_role(&mut self, to: ActorId) {
        self.ensure_is_admin();
        self.get_mut().burners.insert(to);
    }

    #[export]
    pub fn revoke_admin_role(&mut self, from: ActorId) {
        self.ensure_is_admin();
        self.get_mut().admins.remove(&from);
    }

    #[export]
    pub fn revoke_minter_role(&mut self, from: ActorId) {
        self.ensure_is_admin();
        self.get_mut().minters.remove(&from);
    }

    #[export]
    pub fn revoke_burner_role(&mut self, from: ActorId) {
        self.ensure_is_admin();
        self.get_mut().burners.remove(&from);
    }

    #[export]
    pub fn minters(&self) -> Vec<ActorId> {
        self.get().minters.clone().into_iter().collect()
    }

    #[export]
    pub fn burners(&self) -> Vec<ActorId> {
        self.get().burners.clone().into_iter().collect()
    }

    #[export]
    pub fn admins(&self) -> Vec<ActorId> {
        self.get().admins.clone().into_iter().collect()
    }
}

impl ExtendedService {
    fn ensure_is_admin(&self) {
        if !self.get().admins.contains(&msg::source()) {
            panic!("Not admin")
        };
    }
}
