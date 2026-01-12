use extended_vnft_client::vnft::Vnft;
use extended_vnft_client::{ExtendedVnftClient, ExtendedVnftClientCtors, TokenMetadata};
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

    let program_code_id = system.submit_code(extended_vnft::WASM_BINARY);

    let env = GtestEnv::new(system, ADMIN_ID.into());

    let program = env
        .deploy::<extended_vnft_client::ExtendedVnftClientProgram>(
            program_code_id,
            b"salt".to_vec(),
        )
        .new(
            "collection_name".to_string(),
            "collection_symbol".to_string(),
        )
        .await
        .unwrap();

    let mut vnft_client = program.vnft();
    // mint
    let metadata = TokenMetadata {
        name: "token_name".to_string(),
        description: "token_description".to_string(),
        media: "token_media".to_string(),
        reference: "token_reference".to_string(),
    };
    vnft_client.mint(ADMIN_ID.into(), metadata).await.unwrap();
    // check balance
    let balance = vnft_client.balance_of(ADMIN_ID.into()).await.unwrap();
    assert_eq!(balance, 1.into());
    // check token_id
    let token_id = vnft_client.token_id().await.unwrap();
    assert_eq!(token_id, 1.into());
    // check owner
    let actor_id = vnft_client.owner_of(0.into()).await.unwrap();
    assert_eq!(actor_id, ADMIN_ID.into());

    // transfer
    vnft_client
        .transfer(USER_ID[0].into(), 0.into())
        .await
        .unwrap();
    // check owner
    let actor_id = vnft_client.owner_of(0.into()).await.unwrap();
    assert_eq!(actor_id, USER_ID[0].into());

    // approve
    vnft_client
        .approve(USER_ID[1].into(), 0.into())
        .with_actor_id(USER_ID[0].into())
        .await
        .unwrap();

    // transfer from
    vnft_client
        .transfer_from(USER_ID[0].into(), ADMIN_ID.into(), 0.into())
        .with_actor_id(USER_ID[1].into())
        .await
        .unwrap();
    // check owner
    let actor_id = vnft_client.owner_of(0.into()).await.unwrap();
    assert_eq!(actor_id, ADMIN_ID.into());

    // burn
    vnft_client.burn(ADMIN_ID.into(), 0.into()).await.unwrap();
    // check balance
    let balance = vnft_client.balance_of(ADMIN_ID.into()).await.unwrap();
    assert_eq!(balance, 0.into());
    // check owner
    let actor_id = vnft_client.owner_of(0.into()).await.unwrap();
    assert_eq!(actor_id, 0.into());
}

#[tokio::test]
async fn test_grant_role() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=info,sails_rs=debug");
    system.mint_to(ADMIN_ID, DEFAULT_USERS_INITIAL_BALANCE);
    system.mint_to(USER_ID[0], DEFAULT_USERS_INITIAL_BALANCE);
    system.mint_to(USER_ID[1], DEFAULT_USERS_INITIAL_BALANCE);

    let program_code_id = system.submit_code(extended_vnft::WASM_BINARY);

    let env = GtestEnv::new(system, ADMIN_ID.into());

    let program = env
        .deploy::<extended_vnft_client::ExtendedVnftClientProgram>(
            program_code_id,
            b"salt".to_vec(),
        )
        .new(
            "collection_name".to_string(),
            "collection_symbol".to_string(),
        )
        .await
        .unwrap();

    let mut vnft_client = program.vnft();

    // try minter role
    let metadata = TokenMetadata {
        name: "token_name".to_string(),
        description: "token_description".to_string(),
        media: "token_media".to_string(),
        reference: "token_reference".to_string(),
    };
    let res = vnft_client
        .mint(USER_ID[0].into(), metadata)
        .with_actor_id(USER_ID[0].into())
        .await;
    assert!(res.is_err());
    // grant mint role
    vnft_client
        .grant_minter_role(USER_ID[0].into())
        .await
        .unwrap();
    let minters = vnft_client.minters().await.unwrap();
    assert!(minters.contains(&ADMIN_ID.into()));
    assert!(minters.contains(&USER_ID[0].into()));
    vnft_client
        .mint(
            USER_ID[0].into(),
            TokenMetadata {
                name: "token_name".to_string(),
                description: "token_description".to_string(),
                media: "token_media".to_string(),
                reference: "token_reference".to_string(),
            },
        )
        .with_actor_id(USER_ID[0].into())
        .await
        .unwrap();

    let balance = vnft_client.balance_of(USER_ID[0].into()).await.unwrap();
    assert_eq!(balance, 1.into());

    // try burner role
    let res = vnft_client
        .burn(USER_ID[0].into(), 0.into())
        .with_actor_id(USER_ID[0].into())
        .await;
    assert!(res.is_err());
    // grant burn role
    vnft_client
        .grant_burner_role(USER_ID[0].into())
        .await
        .unwrap();
    let burners = vnft_client.burners().await.unwrap();
    assert!(burners.contains(&ADMIN_ID.into()));
    assert!(burners.contains(&USER_ID[0].into()));
    vnft_client
        .burn(USER_ID[0].into(), 0.into())
        .with_actor_id(USER_ID[0].into())
        .await
        .unwrap();

    let balance = vnft_client.balance_of(USER_ID[0].into()).await.unwrap();
    assert_eq!(balance, 0.into());

    // grant admin role
    vnft_client
        .grant_admin_role(USER_ID[0].into())
        .await
        .unwrap();
    let admins = vnft_client.admins().await.unwrap();
    assert!(admins.contains(&ADMIN_ID.into()));
    assert!(admins.contains(&USER_ID[0].into()));
    // revoke roles
    vnft_client
        .revoke_admin_role(USER_ID[0].into())
        .await
        .unwrap();
    let admins = vnft_client.admins().await.unwrap();
    assert_eq!(admins, vec![ADMIN_ID.into()]);
    vnft_client
        .revoke_minter_role(USER_ID[0].into())
        .await
        .unwrap();
    let minters = vnft_client.minters().await.unwrap();
    assert_eq!(minters, vec![ADMIN_ID.into()]);
    vnft_client
        .revoke_burner_role(USER_ID[0].into())
        .await
        .unwrap();
    let burners = vnft_client.burners().await.unwrap();
    assert_eq!(burners, vec![ADMIN_ID.into()]);
}
