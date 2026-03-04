# VFT (Vara Fungible Token)

The VFT program constitutes a fungible token contract that incorporates a complicated role management mechanism. It enables the creation of fungible tokens with configurable parameters, such as the token name, symbol, and decimal precision. The contract implements essential token functionalities, including minting, burning, transferring tokens, and managing allowances, while enforcing stringent role-based access controls to safeguard the system and ensure the proper delegation of authority over these operations.

### 🏗️ Building

```sh
cargo b -r 
```

### ✅ Testing

Run all tests, except `gclient` ones:
```sh
cargo t -r 
```

Run all tests:
```sh
# Download the node binary.
cargo t -r -- --ignored
```

### 📈 Stress test: max unique users (≈2 GiB) — balances only

This repository includes an optional stress test that estimates how many **unique balance holders** can fit into ~2 GiB of program storage (Gear pages, 16 KiB each).

The stress test fills **only** the balances map (and does **not** account for allowances or other state):

```rust
pub type BalancesMap = ShardedMap<NonZeroActorId, NzBalance>;
```

### Running the stress test

The test is guarded by a feature flag and is intentionally **not** part of the default test suite.

```sh
cargo t -r --features stress-tests -- --nocapture
```

### Results (balances-only, ≈2 GiB)

The result depends on the internal amount representation (`LeBytes<N>`). Smaller `amount_*` packs balances more tightly and allows more unique users.

| Amount config | Approx. max unique users (balances-only) |
|---|---:|
| `amount_256` | 28,268,560 |
| `amount_224` | 30,362,944 |
| `amount_192` | 32,247,328 |
| `amount_160` | 34,970,464 |
| `amount_128` | 37,692,976 |
| `amount_96`  | 41,043,616 |
| `amount_80`  | 43,138,000 |

