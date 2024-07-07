use super::vft;
use super::vft::Storage;
use collections::HashSet;
use gstd::msg;
use sails_rtl::{gstd::gservice, prelude::*};
mod funcs;
use crate::services;

#[derive(Default)]
pub struct VftMasterData {
    minters: HashSet<ActorId>,
    burners: HashSet<ActorId>,
    admins: HashSet<ActorId>,
}

static mut VFT_MASTER_DATA: Option<VftMasterData> = None;

#[derive(Encode, Decode, TypeInfo)]
pub enum VftMasterEvent {
    BatchMinted {
        to: Vec<sails_rtl::ActorId>,
        value: Vec<U256>,
    },
    Burned {
        from: sails_rtl::ActorId,
        value: U256,
    },
}
#[derive(Clone)]
pub struct VftMaster {
    vft: vft::Service,
}

impl VftMaster {
    pub fn seed(name: String, symbol: String, decimals: u8) -> Self {
        let admin = msg::source();
        unsafe {
            VFT_MASTER_DATA = Some(VftMasterData {
                admins: [admin].into(),
                minters: [admin].into(),
                burners: [admin].into(),
            });
        };
        VftMaster {
            vft: <vft::Service>::seed(name, symbol, decimals),
        }
    }

    pub fn data_mut(&mut self) -> &'static mut VftMasterData {
        unsafe {
            VFT_MASTER_DATA
                .as_mut()
                .expect("Vft master is not initialized")
        }
    }
    pub fn data(&self) -> &'static VftMasterData {
        unsafe {
            VFT_MASTER_DATA
                .as_ref()
                .expect("Vft master is not initialized")
        }
    }
}

#[gservice(extends = vft::Service, events = VftMasterEvent)]
impl VftMaster {
    pub fn new() -> Self {
        Self {
            vft: vft::Service::new(),
        }
    }
    pub fn batch_mint(&mut self, to: Vec<ActorId>, value: Vec<U256>) -> bool {
        if !self.data().minters.contains(&msg::source()) {
            panic!("Not allowed to mint")
        };

        let mutated = services::utils::panicking(|| {
            funcs::batch_mint(
                &mut Storage::balances(),
                &mut Storage::total_supply(),
                to.clone(),
                value.clone(),
            )
        });
        if mutated {
            let _ = self.notify_on(VftMasterEvent::BatchMinted { to, value });
        }
        mutated
    }

    pub fn burn(&mut self, from: ActorId, value: U256) -> bool {
        if !self.data().burners.contains(&msg::source()) {
            panic!("Not allowed to burn")
        };

        let mutated = services::utils::panicking(|| {
            funcs::burn(
                &mut Storage::balances(),
                &mut Storage::total_supply(),
                from,
                value,
            )
        });
        if mutated {
            let _ = self.notify_on(VftMasterEvent::Burned { from, value });
        }
        mutated
    }

    pub fn grant_minter_role(&mut self, to: ActorId) {
        self.ensure_is_admin();
        self.data_mut().minters.insert(to);
    }
    pub fn grant_burner_role(&mut self, to: ActorId) {
        self.ensure_is_admin();
        self.data_mut().burners.insert(to);
    }

    pub fn revoke_minter_role(&mut self, from: ActorId) {
        self.ensure_is_admin();
        self.data_mut().minters.remove(&from);
    }
    pub fn revoke_burner_role(&mut self, from: ActorId) {
        self.ensure_is_admin();
        self.data_mut().burners.remove(&from);
    }
    pub fn minters(&self) -> Vec<ActorId> {
        self.data().minters.iter().map(|&key| key.clone()).collect()
    }

    pub fn burners(&self) -> Vec<ActorId> {
        self.data().burners.iter().map(|&key| key.clone()).collect()
    }

    pub fn admins(&self) -> Vec<ActorId> {
        self.data().admins.iter().map(|&key| key.clone()).collect()
    }
}

impl VftMaster {
    fn ensure_is_admin(&self) {
        if !self.data().admins.contains(&msg::source()) {
            panic!("Not admin")
        };
    }
}
impl AsRef<vft::Service> for VftMaster {
    fn as_ref(&self) -> &vft::Service {
        &self.vft
    }
}
