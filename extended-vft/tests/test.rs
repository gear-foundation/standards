use extended_vft_client::vft::Vft;
use extended_vft_client::{ExtendedVftClient, ExtendedVftClientCtors};
use sails_rs::gtest::constants::DEFAULT_USERS_INITIAL_BALANCE;
use sails_rs::{client::*, gtest::*};

pub const ADMIN_ID: u64 = 10;
pub const USER_ID: [u64; 2] = [11, 12];

#[tokio::test]
async fn test_basic_function() {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=info,gtest=info,sails_rs=info");
    system.mint_to(ADMIN_ID, DEFAULT_USERS_INITIAL_BALANCE);
    system.mint_to(USER_ID[0], DEFAULT_USERS_INITIAL_BALANCE);
    system.mint_to(USER_ID[1], DEFAULT_USERS_INITIAL_BALANCE);

    let program_code_id = system.submit_code(extended_vft::WASM_BINARY);

    let env = GtestEnv::new(system, ADMIN_ID.into());

    let program = env
        .deploy::<extended_vft_client::ExtendedVftClientProgram>(program_code_id, b"salt".to_vec())
        .new(
            "name".to_string(),
            "symbol".to_string(),
            10,
            None::<Vec<u32>>,
            None::<Vec<u32>>,
        )
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
    system.init_logger_with_default_filter("gwasm=info,gtest=info,sails_rs=info");
    system.mint_to(ADMIN_ID, DEFAULT_USERS_INITIAL_BALANCE);
    system.mint_to(USER_ID[0], DEFAULT_USERS_INITIAL_BALANCE);
    system.mint_to(USER_ID[1], DEFAULT_USERS_INITIAL_BALANCE);

    let program_code_id = system.submit_code(extended_vft::WASM_BINARY);

    let env = GtestEnv::new(system, ADMIN_ID.into());

    let program = env
        .deploy::<extended_vft_client::ExtendedVftClientProgram>(program_code_id, b"salt".to_vec())
        .new(
            "name".to_string(),
            "symbol".to_string(),
            10,
            None::<Vec<u32>>,
            None::<Vec<u32>>,
        )
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

#[cfg(feature = "stress-tests")]
mod stress_tests {
    use super::*;
    use extended_vft_client::vft::Vft;
    use gear_utils::ProgramMemoryDump;

    /// Substring we expect in UserspacePanic when allocator fails in the runtime.
    const OOM_SENTINEL: &str = "memory allocation of 0 bytes failed";

    fn is_oom_debug(e: &impl core::fmt::Debug) -> bool {
        let s = format!("{e:?}");

        // Fast path: sometimes the message is already present as text
        if s.contains(OOM_SENTINEL) {
            return true;
        }

        // Try to extract the byte array: "... UserspacePanic), [112, 97, ...])"
        let start = match s.find('[') {
            Some(i) => i,
            None => return false,
        };
        let end = match s[start..].find(']') {
            Some(j) => start + j,
            None => return false,
        };

        let inside = &s[start + 1..end]; // "112, 97, 110, ..."
        let mut bytes: Vec<u8> = Vec::new();

        for part in inside.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            if let Ok(v) = part.parse::<u8>() {
                bytes.push(v);
            } else {
                // Not a pure byte list
                return false;
            }
        }

        match core::str::from_utf8(&bytes) {
            Ok(text) => text.contains(OOM_SENTINEL),
            Err(_) => false,
        }
    }

    /// Grow balances storage by appending a new shard and immediately allocating it.
    /// Returns `false` if we hit a hard allocator failure (OOM sentinel).
    async fn grow_balances(vft: &mut impl Vft<Env = GtestEnv>, cap: u32) -> bool {
        if let Err(e) = vft.append_balances_shard(cap).await {
            if is_oom_debug(&e) {
                eprintln!("Stopping: allocator failure during alloc_next_balances_shard: {e:?}");
                return false;
            }
            panic!("append_balances_shard failed: {e:?}");
        }

        if let Err(e) = vft.alloc_next_balances_shard().await {
            if is_oom_debug(&e) {
                eprintln!("Stopping: allocator failure during alloc_next_balances_shard: {e:?}");
                return false;
            }
            panic!("alloc_next_balances_shard failed: {e:?}");
        }

        true
    }

    #[tokio::test]
    async fn estimate_max_users_until_2gib_by_batches() {
        // Gear storage page size is 16KiB (GEAR_PAGE_SIZE). Memory dump size grows in these pages.
        let gear_page_size: u64 = 16 * 1024;

        let system = System::new();
        system.init_logger_with_default_filter("gwasm=info,gtest=info,sails_rs=info");
        system.mint_to(ADMIN_ID, DEFAULT_USERS_INITIAL_BALANCE * 1000);

        let program_code_id = system.submit_code(extended_vft::WASM_BINARY);
        let env = GtestEnv::new(system, ADMIN_ID.into());

        let program = env
            .deploy::<extended_vft_client::ExtendedVftClientProgram>(
                program_code_id,
                b"salt".to_vec(),
            )
            .new(
                "name".to_string(),
                "symbol".to_string(),
                10,
                None::<Vec<u32>>,
                None::<Vec<u32>>,
            )
            .await
            .unwrap();

        let mut vft = program.vft();

        // Pre-allocate a few shards to reduce "cold start" variance.
        vft.alloc_next_balances_shard().await.unwrap();
        vft.alloc_next_balances_shard().await.unwrap();
        vft.alloc_next_balances_shard().await.unwrap();

        // Shard capacity used when we need to extend the plan at runtime.
        let cap: u32 = (7 << 15) as u32;

        // --- Baseline dump (state size at time 0) ---
        let before_path = "target/before.dump";
        env.system()
            .get_program(program.id())
            .unwrap()
            .save_memory_dump(before_path);
        let before_pages = ProgramMemoryDump::load_from_file(before_path).pages.len();

        // --- Fill accounts in batches using mint_range(start, count, value) ---
        let mut batch: u32 = 10_000;
        let mut next_id: u64 = 1_000_000;
        let mut total_users: u64 = 0;

        // Throttle expensive dumps.
        let mut last_report_users: u64 = 0;

        loop {
            let minted = match vft.mint_range(next_id, batch, 1u64.into()).await {
                Ok(m) => m,
                Err(e) => {
                    // Hard-stop on allocator failure.
                    if is_oom_debug(&e) {
                        eprintln!("Stopping: allocator failure detected: {e:?}");
                        break;
                    }
                    let msg = format!("{e:?}");
                    // If we ran out of gas, reduce the batch size and retry.
                    if msg.contains("RanOutOfGas") && batch > 100 {
                        batch /= 2;
                        eprintln!("RanOutOfGas -> decreasing batch to {batch}");
                        continue;
                    }

                    panic!("mint_range failed: {msg}");
                }
            };

            if minted == 0 {
                // Likely capacity exhausted => try to grow once.
                if !grow_balances(&mut vft, cap).await {
                    break;
                }

                let minted2 = match vft.mint_range(next_id, batch, 1u64.into()).await {
                    Ok(m) => m,
                    Err(e) => {
                        let msg = format!("{e:?}");
                        if is_oom_debug(&msg) {
                            eprintln!("Stopping: allocator failure detected: {msg}");
                            break;
                        }
                        panic!("mint_range failed after grow: {msg}");
                    }
                };

                if minted2 == 0 {
                    break;
                }

                total_users += minted2 as u64;
                next_id += minted2 as u64;
            } else {
                total_users += minted as u64;
                next_id += minted as u64;

                // Partial batch => grow and continue.
                if minted < batch {
                    if !grow_balances(&mut vft, cap).await {
                        break;
                    }
                }
            }

            // Periodically dump memory and report approximate state usage.
            if total_users >= last_report_users + 200_000 {
                let after_path = "target/after.dump";
                env.system()
                    .get_program(program.id())
                    .unwrap()
                    .save_memory_dump(after_path);
                let after_pages = ProgramMemoryDump::load_from_file(after_path).pages.len();

                let delta_pages = (after_pages - before_pages) as u64;
                let used_bytes = delta_pages * gear_page_size;
                let used_gib = used_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

                eprintln!("users={total_users}, used_bytes={used_bytes}, used_gib={used_gib:.3}");

                last_report_users = total_users;
            }
        }

        eprintln!("Final users minted: {total_users}");
    }
}
