use extended_vnft_wasm::{
    traits::{ExtendedVnftFactory, Vnft},
    ExtendedVnftFactory as Factory, TokenMetadata, Vnft as VftClient,
};
use sails_rs::calls::*;
use sails_rs::gtest::calls::*;

#[tokio::test]
async fn test_basic_function() {
    let program_space = GTestRemoting::new(100.into());
    program_space.system().init_logger();
    let code_id = program_space
        .system()
        .submit_code_file("../target/wasm32-unknown-unknown/release/extended_vnft_wasm.opt.wasm");

    let extended_vnft_factory = Factory::new(program_space.clone());
    let extended_vnft_id = extended_vnft_factory
        .new(
            "collection_name".to_string(),
            "collection_symbol".to_string(),
        )
        .send_recv(code_id, "123")
        .await
        .unwrap();

    let mut client = VftClient::new(program_space);
    // mint
    let metadata = TokenMetadata {
        name: "token_name".to_string(),
        description: "token_description".to_string(),
        media: "token_media".to_string(),
        reference: "token_reference".to_string(),
    };
    client
        .mint(100.into(), metadata)
        .send_recv(extended_vnft_id)
        .await
        .unwrap();
    // check balance
    let balance = client
        .balance_of(100.into())
        .recv(extended_vnft_id)
        .await
        .unwrap();
    assert_eq!(balance, 1.into());
    // check token_id
    let token_id = client.token_id().recv(extended_vnft_id).await.unwrap();
    assert_eq!(token_id, 1.into());
    // check owner
    let actor_id = client
        .owner_of(0.into())
        .recv(extended_vnft_id)
        .await
        .unwrap();
    assert_eq!(actor_id, 100.into());

    // transfer
    client
        .transfer(101.into(), 0.into())
        .send_recv(extended_vnft_id)
        .await
        .unwrap();
    // check owner
    let actor_id = client
        .owner_of(0.into())
        .recv(extended_vnft_id)
        .await
        .unwrap();
    assert_eq!(actor_id, 101.into());

    // approve
    client
        .approve(102.into(), 0.into())
        .with_args(GTestArgs::new(101.into()))
        .send_recv(extended_vnft_id)
        .await
        .unwrap();

    // transfer from
    client
        .transfer_from(101.into(), 100.into(), 0.into())
        .with_args(GTestArgs::new(102.into()))
        .send_recv(extended_vnft_id)
        .await
        .unwrap();
    // check owner
    let actor_id = client
        .owner_of(0.into())
        .recv(extended_vnft_id)
        .await
        .unwrap();
    assert_eq!(actor_id, 100.into());

    // burn
    client
        .burn(100.into(), 0.into())
        .send_recv(extended_vnft_id)
        .await
        .unwrap();
    // check balance
    let balance = client
        .balance_of(100.into())
        .recv(extended_vnft_id)
        .await
        .unwrap();
    assert_eq!(balance, 0.into());
    // check owner
    let actor_id = client
        .owner_of(0.into())
        .recv(extended_vnft_id)
        .await
        .unwrap();
    assert_eq!(actor_id, 0.into());
}

#[tokio::test]
async fn test_grant_role() {
    let program_space = GTestRemoting::new(100.into());
    program_space.system().init_logger();
    let mut client = VftClient::new(program_space.clone());

    let code_id = program_space
        .system()
        .submit_code_file("../target/wasm32-unknown-unknown/release/extended_vnft_wasm.opt.wasm");

    let extended_vft_factory = Factory::new(program_space.clone());
    let extended_vft_id = extended_vft_factory
        .new("name".to_string(), "symbol".to_string())
        .send_recv(code_id, "123")
        .await
        .unwrap();

    // try minter role
    let metadata = TokenMetadata {
        name: "token_name".to_string(),
        description: "token_description".to_string(),
        media: "token_media".to_string(),
        reference: "token_reference".to_string(),
    };
    let res = client
        .mint(101.into(), metadata)
        .with_args(GTestArgs::new(101.into()))
        .send_recv(extended_vft_id)
        .await;
    assert!(res.is_err());
    // grant mint role
    client
        .grant_minter_role(101.into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    let minters = client.minters().recv(extended_vft_id).await.unwrap();
    assert_eq!(minters, vec![100.into(), 101.into()]);
    client
        .mint(
            101.into(),
            TokenMetadata {
                name: "token_name".to_string(),
                description: "token_description".to_string(),
                media: "token_media".to_string(),
                reference: "token_reference".to_string(),
            },
        )
        .with_args(GTestArgs::new(101.into()))
        .send_recv(extended_vft_id)
        .await
        .unwrap();

    let balance = client
        .balance_of(101.into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 1.into());

    // try burner role
    let res = client
        .burn(101.into(), 0.into())
        .with_args(GTestArgs::new(101.into()))
        .send_recv(extended_vft_id)
        .await;
    assert!(res.is_err());
    // grant burn role
    client
        .grant_burner_role(101.into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    let burners = client.burners().recv(extended_vft_id).await.unwrap();
    assert_eq!(burners, vec![100.into(), 101.into()]);
    client
        .burn(101.into(), 0.into())
        .with_args(GTestArgs::new(101.into()))
        .send_recv(extended_vft_id)
        .await
        .unwrap();

    let balance = client
        .balance_of(101.into())
        .recv(extended_vft_id)
        .await
        .unwrap();
    assert_eq!(balance, 0.into());

    // grant admin role
    client
        .grant_admin_role(101.into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    let admins = client.admins().recv(extended_vft_id).await.unwrap();
    assert_eq!(admins, vec![100.into(), 101.into()]);

    // revoke roles
    client
        .revoke_admin_role(101.into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    let admins = client.admins().recv(extended_vft_id).await.unwrap();
    assert_eq!(admins, vec![100.into()]);
    client
        .revoke_minter_role(101.into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    let minters = client.minters().recv(extended_vft_id).await.unwrap();
    assert_eq!(minters, vec![100.into()]);
    client
        .revoke_burner_role(101.into())
        .send_recv(extended_vft_id)
        .await
        .unwrap();
    let burners = client.burners().recv(extended_vft_id).await.unwrap();
    assert_eq!(burners, vec![100.into()]);
}
