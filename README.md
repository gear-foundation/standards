## Standards (Vara Token Programs)

This workspace contains extended implementations of three token standards for the Vara network, built with the [Sails](https://github.com/gear-tech/sails) framework.:

* **extended-vft**: Fungible Token with role-based access control (admins, minters, burners)
* **extended-vmt**: Multiple Token (semi-fungible, ERC-1155-like) with roles and per-token metadata
* **extended-vnft**: Non‑Fungible Token with roles and rich on-chain/off-chain metadata support

Each standard exposes a base `*-service` crate with core storage and logic and an `app` crate that composes and extends it with additional functionality and events. Clients can be generated via `sails-rs` for type-safe contract calls.

### Workspace layout

```
extended-vft/          # VFT program (base + app + client)
extended-vmt/          # VMT program (base + app + client)
extended-vnft/         # VNFT program (base + app + client)
vft-service/           # Base service for VFT
vmt-service/           # Base service for VMT
vnft-service/          # Base service for VNFT
```

### Build

Build all workspace members in release mode:

```bash
cargo build -r
```

Build a specific program (example: extended VFT):

```bash
cargo build -r -p extended-vft
```

Note: `extended-*` crates include the `app` Wasm via a build script; building the top-level crate is sufficient to produce artifacts.

### Test

Unit and regular tests:

```bash
cargo test -r
```

Some integration tests rely on a local node and are marked `ignored`. To run all tests (including `gclient` ones), make sure a node binary is available and then:

```bash
cargo test -r -- --ignored
```

### Roles and permissions (high level)

* **Admins**: Can grant/revoke admin/minter/burner roles
* **Minters**: Can mint tokens
* **Burners**: Can burn tokens

Role checks are enforced in exported methods; unauthorized calls panic with a clear message.

### Deploying on Vara

For a concise end‑to‑end guide to running programs on Vara (building, uploading Wasm, instantiation), follow the official documentation: [Getting started in 5 minutes](https://wiki.vara.network/docs/getting-started-in-5-minutes).

### Public CodeIds

You can instantiate your own token programs using the public CodeIds below. The instantiating account becomes the initial admin/minter/burner where applicable.

> Tip: Click the corresponding **[link]** for **Mainnet** or **Testnet** to open the Gear IDEA portal. In IDEA → **Codes**, paste (or it will already be prefilled, if supported) the CodeId and click **Create Program**.

#### VFT (extended-vft)

* **CodeId**: `0x81663df58f48684923777cd8cf281bfd2e4ee427926abc52a1fcf4ecd41be7ad`

  * Mainnet: [link](https://idea.gear-tech.io/code/0x81663df58f48684923777cd8cf281bfd2e4ee427926abc52a1fcf4ecd41be7ad?node=wss%3A%2F%2Frpc.vara.network)
  * Testnet: [link](https://idea.gear-tech.io/code/0x81663df58f48684923777cd8cf281bfd2e4ee427926abc52a1fcf4ecd41be7ad?node=wss%3A%2F%2Ftestnet.vara.network)
* **Constructor**: `ExtendedVftProgram::new(name, symbol, decimals)`

#### VMT (extended-vmt)

* **CodeId**: `0x3c902523c31f930a4169a5149ff439ec2574a6a6cebe3d6c06742bb254073566`

  * Mainnet: [link](https://idea.gear-tech.io/code/0x3c902523c31f930a4169a5149ff439ec2574a6a6cebe3d6c06742bb254073566?node=wss%3A%2F%2Frpc.vara.network)
  * Testnet: [link](https://idea.gear-tech.io/code/0x3c902523c31f930a4169a5149ff439ec2574a6a6cebe3d6c06742bb254073566?node=wss%3A%2F%2Ftestnet.vara.network)
* **Constructor**: `ExtendedVmtProgram::new(/* required params */)`

#### VNFT (extended-vnft)

* **CodeId**: `0xbba6636d3bec4f203d4ae9b58d9bc9995c7aa20344028001f22dceb43911afad`

  * Mainnet: [link](https://idea.gear-tech.io/code/0xbba6636d3bec4f203d4ae9b58d9bc9995c7aa20344028001f22dceb43911afad?node=wss%3A%2F%2Frpc.vara.network)
  * Testnet: [link](https://idea.gear-tech.io/code/0xbba6636d3bec4f203d4ae9b58d9bc9995c7aa20344028001f22dceb43911afad?node=wss%3A%2F%2Ftestnet.vara.network)
* **Constructor**: `ExtendedVnftProgram::new(/* required params */)`
