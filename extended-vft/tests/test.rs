use extended_vft_client::vft::Vft;
use extended_vft_client::{ExtendedVftClient, ExtendedVftClientCtors};
use sails_rs::gtest::constants::DEFAULT_USERS_INITIAL_BALANCE;
use sails_rs::{client::*, gtest::*};

pub const ADMIN_ID: u64 = 10;
pub const USER_ID: [u64; 2] = [11, 12];

#[tokio::test]
async fn test_basic_function() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=info,sails_rs=debug");
    system.mint_to(ADMIN_ID, DEFAULT_USERS_INITIAL_BALANCE);
    system.mint_to(USER_ID[0], DEFAULT_USERS_INITIAL_BALANCE);
    system.mint_to(USER_ID[1], DEFAULT_USERS_INITIAL_BALANCE);

    let program_code_id = system.submit_code(extended_vft::WASM_BINARY);

    let env = GtestEnv::new(system, ADMIN_ID.into());

    let program = env
        .deploy::<extended_vft_client::ExtendedVftClientProgram>(program_code_id, b"salt".to_vec())
        .new("name".to_string(), "symbol".to_string(), 10)
        .await
        .unwrap();

    let mut vft_client = program.vft();

    // mint
    vft_client
        .mint(ADMIN_ID.into(), 1_000.into())
        .await
        .unwrap();
    // check balance
    let balance = vft_client.balance_of(ADMIN_ID.into()).await.unwrap();
    assert_eq!(balance, 1_000.into());

    // burn
    vft_client.burn(ADMIN_ID.into(), 100.into()).await.unwrap();
    // check balance
    let balance = vft_client.balance_of(ADMIN_ID.into()).await.unwrap();
    assert_eq!(balance, 900.into());

    // transfer
    vft_client
        .transfer(USER_ID[0].into(), 100.into())
        .await
        .unwrap();
    // check balance
    let balance = vft_client.balance_of(ADMIN_ID.into()).await.unwrap();
    assert_eq!(balance, 800.into());
    let balance = vft_client.balance_of(USER_ID[0].into()).await.unwrap();
    assert_eq!(balance, 100.into());

    // approve
    vft_client
        .approve(USER_ID[1].into(), 100.into())
        .await
        .unwrap();
    // check balance
    let balance = vft_client.balance_of(ADMIN_ID.into()).await.unwrap();
    assert_eq!(balance, 800.into());
    let balance = vft_client.balance_of(USER_ID[1].into()).await.unwrap();
    assert_eq!(balance, 0.into());
    // transfer from
    vft_client
        .transfer_from(ADMIN_ID.into(), USER_ID[0].into(), 100.into())
        .await
        .unwrap();
    // check balance
    let balance = vft_client.balance_of(ADMIN_ID.into()).await.unwrap();
    assert_eq!(balance, 700.into());
    let balance = vft_client.balance_of(USER_ID[0].into()).await.unwrap();
    assert_eq!(balance, 200.into());
    let balance = vft_client.balance_of(USER_ID[1].into()).await.unwrap();
    assert_eq!(balance, 0.into());
}

#[tokio::test]
async fn test_grant_role() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=info,sails_rs=debug");
    system.mint_to(ADMIN_ID, DEFAULT_USERS_INITIAL_BALANCE);
    system.mint_to(USER_ID[0], DEFAULT_USERS_INITIAL_BALANCE);
    system.mint_to(USER_ID[1], DEFAULT_USERS_INITIAL_BALANCE);

    let program_code_id = system.submit_code(extended_vft::WASM_BINARY);

    let env = GtestEnv::new(system, ADMIN_ID.into());

    let program = env
        .deploy::<extended_vft_client::ExtendedVftClientProgram>(program_code_id, b"salt".to_vec())
        .new("name".to_string(), "symbol".to_string(), 10)
        .await
        .unwrap();

    let mut vft_client = program.vft();

    // try minter role
    let res = vft_client
        .mint(USER_ID[0].into(), 1_000.into())
        .with_actor_id(USER_ID[0].into())
        .await;
    assert!(res.is_err());
    // grant mint role
    vft_client
        .grant_minter_role(USER_ID[0].into())
        .await
        .unwrap();
    let minters = vft_client.minters().await.unwrap();
    assert!(minters.contains(&ADMIN_ID.into()));
    assert!(minters.contains(&USER_ID[0].into()));
    let res = vft_client
        .mint(USER_ID[0].into(), 1_000.into())
        .with_actor_id(USER_ID[0].into())
        .await
        .unwrap();
    assert!(res);
    let balance = vft_client.balance_of(USER_ID[0].into()).await.unwrap();
    assert_eq!(balance, 1_000.into());

    // try burner role
    let res = vft_client
        .burn(USER_ID[0].into(), 1_000.into())
        .with_actor_id(USER_ID[0].into())
        .await;
    assert!(res.is_err());
    // grant burner role
    vft_client
        .grant_burner_role(USER_ID[0].into())
        .await
        .unwrap();
    let burners = vft_client.burners().await.unwrap();
    assert!(burners.contains(&ADMIN_ID.into()));
    assert!(burners.contains(&USER_ID[0].into()));
    let res = vft_client
        .burn(USER_ID[0].into(), 1_000.into())
        .with_actor_id(USER_ID[0].into())
        .await
        .unwrap();
    assert!(res);
    let balance = vft_client.balance_of(USER_ID[0].into()).await.unwrap();
    assert_eq!(balance, 0.into());

    // grant admin role
    vft_client
        .grant_admin_role(USER_ID[0].into())
        .await
        .unwrap();
    let admins = vft_client.admins().await.unwrap();
    assert!(admins.contains(&ADMIN_ID.into()));
    assert!(admins.contains(&USER_ID[0].into()));
    // revoke roles
    vft_client
        .revoke_admin_role(USER_ID[0].into())
        .await
        .unwrap();
    let admins = vft_client.admins().await.unwrap();
    assert_eq!(admins, vec![ADMIN_ID.into()]);
    vft_client
        .revoke_minter_role(USER_ID[0].into())
        .await
        .unwrap();
    let minters = vft_client.minters().await.unwrap();
    assert_eq!(minters, vec![ADMIN_ID.into()]);
    vft_client
        .revoke_burner_role(USER_ID[0].into())
        .await
        .unwrap();
    let burners = vft_client.burners().await.unwrap();
    assert_eq!(burners, vec![ADMIN_ID.into()]);
}
