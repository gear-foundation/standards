use erc20::services::admin::utils::Role;
use gstd::{ActorId, Encode};
use gtest::System;
use sails_rtl::U256;

mod utils;
use utils::*;

#[test]
fn test_roles() {
    let sys = System::new();
    sys.init_logger();

    let ft = init(&sys);
    let admin_user: ActorId = USERS[0].into();
    let user: ActorId = USERS[1].into();

    // failed mint
    let value: U256 = 1_000.into();
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "Mint", payload: (admin_user, value));
    assert!(res.main_failed());

    // failed grant role
    let res = send_request!(ft: ft, user: USERS[1], service_name: "Admin", action: "GrantRole", payload: (user, Role::Minter));
    assert!(res.main_failed());

    // success grant role
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "GrantRole", payload: (admin_user, Role::Minter));
    assert!(!res.main_failed());

    // success mint
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "Mint", payload: (admin_user, value));
    assert!(!res.main_failed());

    // remove role
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "RemoveRole", payload: (admin_user, Role::Minter));
    assert!(!res.main_failed());

    // failed mint
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "Burn", payload: (admin_user, value));
    assert!(res.main_failed());

    // failed burn
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "Burn", payload: (admin_user, value));
    assert!(res.main_failed());

    // grant role
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "GrantRole", payload: (admin_user, Role::Burner));
    assert!(!res.main_failed());

    // success burn
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "Burn", payload: (admin_user, value / 2));
    assert!(!res.main_failed());

    // remove role
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "RemoveRole", payload: (admin_user, Role::Burner));
    assert!(!res.main_failed());

    // failed burn
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "Burn", payload: (admin_user, value));
    assert!(res.main_failed());

    // failed pause
    let res = send_request!(ft: ft, user: USERS[1], service_name: "Pausable", action: "Pause", payload: ());
    assert!(res.main_failed());

    // delegate admin
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Pausable", action: "DelegateAdmin", payload: (user));
    assert!(!res.main_failed());

    // success pause
    let res = send_request!(ft: ft, user: USERS[1], service_name: "Pausable", action: "Pause", payload: ());
    assert!(!res.main_failed());

    // failed kill
    let res = send_request!(ft: ft, user: USERS[1], service_name: "Admin", action: "Kill", payload: (user));
    assert!(res.main_failed());

    // success kill
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "Kill", payload: (user));
    assert!(!res.main_failed());
}

#[test]
fn test_pausable() {
    let sys = System::new();
    sys.init_logger();

    let ft = init(&sys);
    let admin_user: ActorId = USERS[0].into();

    // success grant role
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "GrantRole", payload: (admin_user, Role::Minter));
    assert!(!res.main_failed());

    // success grant role
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "GrantRole", payload: (admin_user, Role::Burner));
    assert!(!res.main_failed());

    // success pause
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Pausable", action: "Pause", payload: ());
    assert!(!res.main_failed());

    // failed grant role
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "GrantRole", payload: (admin_user, Role::Minter));
    assert!(res.main_failed());

    // failed mint
    let value: U256 = 1_000.into();
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "Mint", payload: (admin_user, value));
    assert!(res.main_failed());

    // failed burn
    let res = send_request!(ft: ft, user: USERS[0], service_name: "Admin", action: "Burn", payload: (admin_user, value));
    assert!(res.main_failed());
}
