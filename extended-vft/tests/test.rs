use extended_vft_client::{
    traits::{ExtendedVftFactory, Vft},
    ExtendedVftFactory as Factory, Vft as VftClient,
};
use sails_rs::calls::*;
use sails_rs::collections::HashMap;
use sails_rs::gtest::{calls::*, System};
use sails_rs::{ActorId, U256};
use std::mem;

pub const ADMIN_ID: u64 = 10;
pub const USER_ID: [u64; 2] = [11, 12];

#[tokio::test]
async fn test_basic_function() {
    let system = System::new();
    system.init_logger();
    system.mint_to(ADMIN_ID, 100_000_000_000_000);
    system.mint_to(USER_ID[0], 100_000_000_000_000);
    system.mint_to(USER_ID[1], 100_000_000_000_000);

    let program_space = GTestRemoting::new(system, ADMIN_ID.into());
    let code_id = program_space
        .system()
        .submit_code_file("../target/wasm32-unknown-unknown/release/extended_vft.opt.wasm");

    let extended_vft_factory = Factory::new(program_space.clone());
    let extended_vft_id = extended_vft_factory
        .new("name".to_string(), "symbol".to_string(), 10)
        .send_recv(code_id, "123")
        .await
        .unwrap();

    let mut client = VftClient::new(program_space);
    // mint
    client
        .mint(ADMIN_ID.into(), 1_000.into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    // check balance
    let balance = client
        .balance_of(ADMIN_ID.into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 1_000.into());

    // burn
    client
        .burn(ADMIN_ID.into(), 100.into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    // check balance
    let balance = client
        .balance_of(ADMIN_ID.into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 900.into());

    // transfer
    client
        .transfer(USER_ID[0].into(), 100.into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    // check balance
    let balance = client
        .balance_of(ADMIN_ID.into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 800.into());
    let balance = client
        .balance_of(USER_ID[0].into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 100.into());

    // approve
    client
        .approve(USER_ID[1].into(), 100.into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    // check balance
    let balance = client
        .balance_of(ADMIN_ID.into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 800.into());
    let balance = client
        .balance_of(USER_ID[1].into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 0.into());
    // transfer from
    client
        .transfer_from(ADMIN_ID.into(), USER_ID[0].into(), 100.into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    // check balance
    let balance = client
        .balance_of(ADMIN_ID.into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 700.into());
    let balance = client
        .balance_of(USER_ID[0].into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 200.into());
    let balance = client
        .balance_of(USER_ID[1].into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 0.into());
}

#[tokio::test]
async fn test_grant_role() {
    let system = System::new();
    system.init_logger();
    system.mint_to(ADMIN_ID, 100_000_000_000_000);
    system.mint_to(USER_ID[0], 100_000_000_000_000);
    system.mint_to(USER_ID[1], 100_000_000_000_000);

    let program_space = GTestRemoting::new(system, ADMIN_ID.into());
    let mut client = VftClient::new(program_space.clone());

    let code_id = program_space
        .system()
        .submit_code_file("../target/wasm32-unknown-unknown/release/extended_vft.opt.wasm");

    let extended_vft_factory = Factory::new(program_space.clone());
    let extended_vft_id = extended_vft_factory
        .new("name".to_string(), "symbol".to_string(), 10)
        .send_recv(code_id, "123")
        .await
        .unwrap();

    // try minter role
    let res = client
        .mint(USER_ID[0].into(), 1_000.into())
        .with_args(GTestArgs::new(USER_ID[0].into()))
        .send_recv(extended_vft_id)
        .await;
    assert!(res.is_err());
    // grant mint role
    client
        .grant_minter_role(USER_ID[0].into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    let minters = client.minters().recv(extended_vft_id).await.unwrap();
    assert!(minters.contains(&ADMIN_ID.into()));
    assert!(minters.contains(&USER_ID[0].into()));
    let res = client
        .mint(USER_ID[0].into(), 1_000.into())
        .with_args(GTestArgs::new(USER_ID[0].into()))
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    assert!(res);
    let balance = client
        .balance_of(USER_ID[0].into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 1_000.into());

    // try burner role
    let res = client
        .burn(USER_ID[0].into(), 1_000.into())
        .with_args(GTestArgs::new(USER_ID[0].into()))
        .send_recv(extended_vft_id)
        .await;
    assert!(res.is_err());
    // grant burner role
    client
        .grant_burner_role(USER_ID[0].into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    let burners = client.burners().recv(extended_vft_id).await.unwrap();
    assert!(burners.contains(&ADMIN_ID.into()));
    assert!(burners.contains(&USER_ID[0].into()));
    let res = client
        .burn(USER_ID[0].into(), 1_000.into())
        .with_args(GTestArgs::new(USER_ID[0].into()))
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    assert!(res);
    let balance = client
        .balance_of(USER_ID[0].into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 0.into());

    // grant admin role
    client
        .grant_admin_role(USER_ID[0].into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    let admins = client.admins().recv(extended_vft_id).await.unwrap();
    assert!(admins.contains(&ADMIN_ID.into()));
    assert!(admins.contains(&USER_ID[0].into()));
    // revoke roles
    client
        .revoke_admin_role(USER_ID[0].into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    let admins = client.admins().recv(extended_vft_id).await.unwrap();
    assert_eq!(admins, vec![ADMIN_ID.into()]);
    client
        .revoke_minter_role(USER_ID[0].into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    let minters = client.minters().recv(extended_vft_id).await.unwrap();
    assert_eq!(minters, vec![ADMIN_ID.into()]);
    client
        .revoke_burner_role(USER_ID[0].into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    let burners = client.burners().recv(extended_vft_id).await.unwrap();
    assert_eq!(burners, vec![ADMIN_ID.into()]);
}

#[tokio::test]
async fn test_memory_allocation() {
    let system = System::new();
    system.init_logger();
    system.mint_to(ADMIN_ID, 1_000_000_000_000_000_000);

    let program_space = GTestRemoting::new(system, ADMIN_ID.into());
    let code_id = program_space
        .system()
        .submit_code_file("../target/wasm32-unknown-unknown/release/extended_vft.opt.wasm");

    let extended_vft_factory = Factory::new(program_space.clone());
    let extended_vft_id = extended_vft_factory
        .new("name".to_string(), "symbol".to_string(), 10)
        .send_recv(code_id, "123")
        .await
        .unwrap();

    let mut client = VftClient::new(program_space.clone());

    let mut user_id: u64 = 11;

    let key_size = mem::size_of::<ActorId>();
    let value_size = mem::size_of::<U256>();
    let entry_size = key_size + value_size;

    loop {
        client
            .mint(user_id.into(), 10.into())
            .send_recv(extended_vft_id)
            .await
            .unwrap();
        if user_id % 25_000 == 0 {
            println!("\nUSER ID {:?}", user_id);
            let balances_capacity = client
                .balances_capacity()
                .recv(extended_vft_id)
                .await
                .unwrap();

            let balances_capacity_in_bytes = balances_capacity as usize * entry_size;

            println!("Balances capacity (elements): {:?}", balances_capacity);
            println!(
                "Balances capacity (bytes): {:?}",
                balances_capacity_in_bytes
            );
            if user_id as u128 + 30_000 > balances_capacity {
                client
                    .reserve_capacity(100_000 as u128, 0)
                    .send_recv(extended_vft_id)
                    .await
                    .unwrap();
            }
        }
        user_id += 1;
    }
}

fn hash_map_memory_usage<K, V>(map: &HashMap<K, V>) -> usize {
    let num_buckets = map.capacity();
    let static_size = mem::size_of::<HashMap<K, V>>();
    let bucket_size = mem::size_of::<(K, V)>() * num_buckets;
    static_size + bucket_size
}
