# ADR 002 - Sovereign IBC Light Client Implementation

## Changelog

- 2024-04-18: Drafted

## Status

Implemented (v1)

## Table of Contents

- [ADR 002 - Sovereign IBC Light Client Implementation](#adr-002---sovereign-ibc-light-client-implementation)
  - [Changelog](#changelog)
  - [Status](#status)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [Project Structure](#project-structure)
  - [Top-level Structs](#top-level-structs)
    - [ClientState](#clientstate)
    - [ConsensusState](#consensusstate)
    - [Header](#header)
    - [Misbehaviour](#misbehaviour)
  - [Underlying Structs](#underlying-structs)
    - [SovereignClientParams](#sovereignclientparams)
    - [TendermintClientParams](#tendermintclientparams)
    - [SovereignConsensusParams](#sovereignconsensusparams)
    - [TendermintConsensusParams](#tendermintconsensusparams)
  - [Light Client Implementation](#light-client-implementation)
    - [ClientStateCommon methods](#clientstatecommon-methods)
    - [ClientStateValidation methods](#clientstatevalidation-methods)
    - [ClientStateExecution methods](#clientstateexecution-methods)
  - [CosmWasm Contract Implementation](#cosmwasm-contract-implementation)
  - [References](#references)

## Overview

This ADR describes our approach for implementing the Sovereign IBC light client.
It reflects the latest work done in collaboration with the Sovereign Labs team,
and describes the current state of the Sovereign IBC light client, serving as
its first version. The primary objective here is to share knowledge, explain
design choices, and provides insights into the reasoning behind our verification
logic.

The Sovereign IBC light client aims to establish IBC interoperability between
Sovereign SDK rollups and Cosmos SDK chains. To achieve this, we opted to
develop the light client in Rust, leveraging the common Rust foundation of both
Sovereign SDK and `ibc-rs`. This not only ensures compatibility but also lays
the groundwork for potentially reusing the light client implementation for IBC
integration between Sovereign rollups and projects beyond the Cosmos ecosystem.

For integrating with Cosmos chains, we needed a light client that was compatible
with the Go environment. Fortunately, the Wasm-enabled version of `ibc-go`
offers this capability, allowing us to deploy a Rust-powered light client on
Cosmos chains as a CosmWasm contract. We can then interact with this contract
through the `08-wasm` proxy light client. Our initial target for this
integration is the [Wasm-enabled
v7.3](https://github.com/cosmos/ibc-go/tree/08-wasm/release/v0.1.x%2Bibc-go-v7.3.x-wasmvm-v1.5.x)
on `ibc-go`.

Therefore, our approach involves developing a Rust-based Sovereign light client
that can function as a CosmWasm contract on Cosmos chains, facilitating the
first IBC connection to Sovereign SDK rollups.

## Project Structure

The foundation of our light client is built upon Protocol Buffer definitions,
their Rust representations, and domain types. These components serve as the
essential data structures necessary for implementing a light client using
`ibc-rs` APIs, which are sourced from the
[`ibc-core-client-context`](https://github.com/cosmos/ibc-rs/tree/main/ibc-core/ics02-client/context)
crate. With these elements in place, we can integrate the client into the
CosmWasm storage, proceed with contract implementation, and compile it into a
Wasm file.

We maintain a concise crate named `sov-ibc-proto` specifically designed for Rust
implementations of `.proto` files living under `proto` directory. This library
aids in easier integration with associated projects like [Hermes
SDK](https://github.com/informalsystems/hermes-sdk) and
[`sov-rollup-starter`](https://github.com/informalsystems/sov-rollup-starter)
(For more details, see the
[README](https://github.com/informalsystems/sovereign-ibc/blob/97d81ce3aa3f4e84432e7803f3534f608ef049fe/proto/README.md)).
All code relevant to light clients is organized under the `clients` directory.
Our codebase organization closely aligns with the structure of `ibc-rs`, with
domain types housed in separate crates, catering to a diverse range of consumer
groups including relayers and IBC modules. Notably, we have made a clear
distinction between Rollup-specific domain types and those relevant to data
availability (DA), paving the way for future development of light clients for
rollups operating at different DA layers. Presently, we offer the implementation
of one light client specifically tailored for Sovereign rollups on Celestia DA.

Here is the list of crates:

1. [**`sov-ibc-proto`**](https://github.com/informalsystems/sovereign-ibc/blob/97d81ce3aa3f4e84432e7803f3534f608ef049fe/proto/README.md):
   Provides Rust implementation of `.proto` definitions.
2. [**`sov-client-types`**](https://github.com/informalsystems/sovereign-ibc/blob/a9aaa80c4fe7b21fa777ae2a186838aac1fed68c/clients/sov-types):
   Contains domain types specific to rollups, agnostic to DA.
3. [**`sov-client-celestia-types`**](https://github.com/informalsystems/sovereign-ibc/tree/a9aaa80c4fe7b21fa777ae2a186838aac1fed68c/clients/sov-celestia/types):
   Includes Celestia-specific domain types.
4. [**`sov-client-celestia`**](https://github.com/informalsystems/sovereign-ibc/tree/a9aaa80c4fe7b21fa777ae2a186838aac1fed68c/clients/sov-celestia):
   Implements the light client for rollups operating on Celestia.
5. [**`sov-client-celestia-cw`**](https://github.com/informalsystems/sovereign-ibc/blob/8fe4fa1cefbc9b125a2a73e82166263755430ce9/clients/sov-celestia/cw-contract):
   Implements the CosmWasm contract for the `sov-celestia` client.

## Top-level Structs

In this section we go over the data structure definitions utilized in the
Sovereign IBC light clients. We are focusing here on the top-level structs, each
containing two fields: one for rollup-specific parameters and the other for DA.

This architecture enables being generic over DA-relevant parameters, while the
rollup-specific field is shared among all Sovereign SDK rollups. This separation
aims for clarity and modularity and it is crucial for accommodating diverse
Sovereign light client implementations in the future.

### ClientState

The `ClientState` is defined as follows:

```rust
pub struct ClientState<Da> {
    pub sovereign_params: SovereignClientParams,
    pub da_params: Da,
}
```

### ConsensusState

A similar delineation is present for Sovereign's `ConsensusState`:

```rust
pub struct ConsensusState<Da> {
    pub sovereign_params: SovereignConsensusParams,
    pub da_params: Da,
}
```

### Header

As well, this is the same for the `Header` that will be included as the
`client_message` field in client updates. It comprises two fields:

```rust
pub struct Header<H> {
    pub aggregated_proof: AggregatedProof,
    pub da_header: H,
}
```

The `aggregated_proof` serves as the Zero-Knowledge (ZK) proof for the execution
layer. In a sense, it acts equivalent to rollup's Header within the context of
IBC. The `da_header` focuses on the consensus layer, embodying the core Header
of the Celestia in our case. To grasp the structure of the aggregated proof more
clearly, consider the following representation. This structure reflects the
latest iteration finalized in collaboration with Sovereign Labs, crafted to meet
the requirements of the ZK prover.

```rust
pub struct AggregatedProof {
    pub public_data: AggregatedProofPublicData,
    pub serialized_proof: SerializedAggregatedProof,
}

pub struct AggregatedProofPublicData {
    pub validity_conditions: Vec<ValidityCondition>,
    pub initial_slot_number: SlotNumber,
    pub final_slot_number: SlotNumber,
    pub genesis_state_root: Root,
    pub initial_state_root: Root,
    pub final_state_root: Root,
    pub initial_slot_hash: Vec<u8>,
    pub final_slot_hash: Vec<u8>,
    pub code_commitment: CodeCommitment,
}

pub struct SerializedAggregatedProof(Vec<u8>);
```

### Misbehaviour

Given the Header, the `Misbehaviour` struct takes the same field to the
Tendermint clients as below:

```rust
pub struct Misbehaviour<H> {
    client_id: ClientId,
    header_1: Box<Header<H>>,
    header_2: Box<Header<H>>,
}
```

## Underlying Structs

Let's go through the underlying fields in the `ClientState`, starting with the
rollup-specific parameters grouped under the `SovereignClientParams` struct.

### SovereignClientParams

Take a moment to familiarize yourself with the following overview of the struct.
We will dive into the specifics and touch on some key facts about rollup
functionality afterward.

```rust
pub struct SovereignClientParams {
    /// The height of the DA layer at which the rollup is initialized.
    pub genesis_da_height: Height,
    /// The genesis state root, which is unique to each rollup.
    pub genesis_state_root: Root,
    /// The code commitment of the rollup's software, which is the output
    /// commitment of the ZK circuit.
    pub code_commitment: CodeCommitment,
    /// The trusting period is the period in which headers can be verified.
    pub trusting_period: Duration,
    /// The frozen height indicates whether the client is frozen.
    pub frozen_height: Option<Height>,
    /// The latest trusted height of the rollup.
    pub latest_height: Height,
    /// The path to the location in store where the upgraded client and
    /// consensus states are stored.
    pub upgrade_path: UpgradePath,
}
```

1. **genesis_da_height**

    One notable property is that a rollup can commence at a DA height other than
    zero. This scenario is quite common, implying that there will often be a
    height offset between the DA layer and the rollup. In the Sovereign SDK
    system, height is represented by slot numbers. Therefore, the rollup's slot
    number equals the DA height minus the genesis height. For example, if the
    rollup began at DA layer block 1000, slot number 50 would correspond to DA
    layer block 1050. This height difference is expected to remain constant over
    time.

    As a result, the `genesis_da_height` field has been added into the struct,
    which is initialized during client creation. It can only be modified through
    a client upgrade process. This is permitted because in Cosmos chains, the
    `revision_height` resets during upgrades, although it remains unchanged
    during client recoveries. Speaking of which, recoveries occur when we need
    to re-activate a frozen or expired client, typically through a governance
    mechanism.

    The genesis height can be retrieved from the rollup’s `config.toml` file.
    Additionally, the `sov-ibc` module may expose this parameter via an RPC
    method once it becomes available under the `sov-chain-state` (this request
    has been made to Sovereign Labs, thought for a different reason which we
    will elaborate on later)

2. **genesis_state_root**

    Rollups do not have validator sets, which means we cannot perform signature
    verification like we do for Tendermint headers. This presents a challenge as
    we need a method to distinguish between different rollups. Otherwise, a
    valid aggregated proof from rollup (A) could be used to update the client of
    rollup (B). In the case of Tendermint clients, verifying signatures of
    validators is the step ensuring us that a header belongs to the correct
    chain, thereby preventing updates from headers of other chains. Furthermore,
    Sovereign rollups currently lack any form of identifiers. While there might
    be plans to introduce rollup identifiers using human-readable strings, these
    would primarily be for use by wallet integrations and cannot be relied upon
    to uniquely identify or distinguish one rollup from another or forks.

    The `genesis_state_root` parameter is expected to be unique for rollups, and
    it is initialized during client creation. This value can be obtained from
    the `ledger_getAggregatedProof` RPC method along with the other aggregated
    proof data. However, it's worth noting that there is a nuanced assumption
    here: we are assuming that rollup operators are honest, which ensures that
    the genesis state roots remain unique. However, there's a potential risk
    that a maliciously modified rollup could tamper with this value by altering
    its software. This is why we also need the `code_commitment` field as well.

3. **code_commitment**

    The `code_commitment` is generated as the output of the rollup zk-circuit
    compilation. This means that different execution logics or, in a broader
    sense, different versions of rollups should result in distinct code
    commitments. Alongside the `genesis_state_root`, the `code_commitment`
    serves as the second parameter that helps link an incoming aggregated proof
    to a specific rollup, ensuring uniqueness and integrity. Similarly, this
    field should also be initialized during client creation, and is only
    permitted to change with a client upgrade. And not during recoveries.

4. **trusting_period**

    Initially, the `trusting_period` was solely under the `da_params` of the
    `ClientState`, but it has been elevated to a rollup-wide necessity. This
    shift is motivated by potential emergency scenarios that might necessitate a
    rollup halt, affecting clients on counterpart chains as well.

    Given that the client verifies both the core header of the DA and the
    aggregated proof simultaneously during each update, having separate trusting
    periods for each doesn't align logically. Instead, when configuring this
    period, we must account for both the DA layer and the rollup, ensuring that
    the client's trusting duration is maximum acceptable amount. If, for
    instance, the rollup requires a shorter trusting window, it dictates when
    the client expires. This parameter is determined by relayers/users and can
    be updated during both client recoveries and upgrades.

5. **frozen_height**

    Currently, the Sovereign light client can only be frozen due to misbehavior
    in the DA layer. It's important to note that misbehavior, in the context of
    the rollup, doesn't have the same implications. Here, misbehavior refers to
    a scenario where the passed aggregated proof is initially validated by the
    verifier but is later discovered to be incorrect. Such a case essentially
    invalidates the credibility of the integrated ZK verifier into the light
    client.

    At present, we haven't identified any specific potential misbehavior.
    However, maintaining this field as a rollup-wide parameter allows for future
    investigations that may uncover potential scenarios needing attention.
    Additionally, different rollup nodes might benefit from customized
    executions, making it more sensible to have this parameter rollup-wide.

    A small note regarding this field: When the client is frozen, `ibc-rs` sets
    the height to `Height::new(0, 1)`, following the same logic as in `ibc-go`.

6. **latest_height**

    This field signifies the most recent trusted height of the client. Here,
    "height" refers to the slot number since we're basically tracking the
    rollup's state. However, we stick to using "height" for naming and type due
    to being within the IBC boundary and for consistency with `ibc-rs`. This is
    more efficient, considering that `ibc-rs` mandates the use of the `Height`
    type in underlying implementations.

    As a result, incoming height values, labeled as `SlotNumber` in the
    Sovereign SDK system, and of type `u64`, should undergo conversion to the
    `Height` type.

7. **upgrade_path**

    This field should be set during client creation and remains unchanged
    throughout the client's lifecycle.

    One notable distinction from Tendermint light clients is that we don't
    require a vector of paths (i.e., two paths). Instead, by having a single
    path, we can subsequently locate upgraded client and consensus states in the
    rollup store. Thus, the type of this field is `String` rather than
    `Vec<String>`, as seen in Tendermint clients.

    It is important to highlight that this field is rollup-specific. All IBC
    states are stored within the rollup's database and retrieved from there.

In the next section, we move on to reviewing factors that are used for Celestia
core header verification. The DA relevant variables.

### TendermintClientParams

Celestia introduces an `ExtendedHeader` schema with an additional `dah` field
compared to a standard Tendermint Header.

```rust
pub struct ExtendedHeader {
    pub header: Header,
    pub commit: Commit,
    pub validator_set: ValidatorSet,
    pub dah: DataAvailabilityHeader,
}
```

In the Sovereign SDK rollup, the `dah` field undergoes comprehensive validation
as part of processing each Celestia block. Because the rollup's execution logic
is verifiable through ZK proofs, our light client doesn't need to verify the
Data Availability Header itself. Instead, it delegates this check to the rollup
[`DaVerifier`](https://github.com/informalsystems/sovereign-sdk-wip/blob/c5820ff37e0614887222fe2393a2362ae07fdd2c/adapters/celestia/src/verifier/mod.rs#L184)
and relies on the aggregated proof for coverage. Consequently, the light client
focuses solely on validating the core header, as the rollups do not verify the
validators' signatures that are embedded in the core header.

This approach allows it to work with a normal Tendermint Header, making the IBC
Header struct from ICS-07 reusable. Thus, we use the [ICS-07
Header](https://github.com/cosmos/ibc-rs/blob/f7a3c2b23f42977392b2f2ca908cc98d00765ff5/ibc-clients/ics07-tendermint/types/src/header.rs#L29-L35)
types for the `da_header` field in the aforementioned [Header](#header) struct.

Given this rationale, let's take a closer look to the type of the `da_params` in
the `ClientState`. Namely, the `TendermintClientParams` struct:

```rust
pub struct TendermintClientParams {
    pub chain_id: ChainId,
    pub trust_level: TrustThreshold,
    pub unbonding_period: Duration,
    pub max_clock_drift: Duration,
}
```

1. **`chain_id`**: Represents the identifier of the chain to which the
   Tendermint header belongs.
2. **`trust_level`**: Defines the threshold of trust and helps in determining
   the level of confidence needed before accepting headers for further
   processing.
3. **`unbonding_period`**: Specifies the duration for which bonded validators
   remain unbonded after initiating an unbonding process.
4. **`max_clock_drift`**: Sets the maximum allowable time difference between the
   local clock and the timestamp of a Tendermint header.

Together, these fields plus the `trusting_period` under the
[`SovereignClientParams`](#sovereignclientparams) constitute all the necessary
information to perform a comprehensive validation on a Tendermint header.

It's important to note that within the Sovereign SDK context, we use Jellyfish
Merkle Tree (JMT) proofs. Unlike Tendermint clients, the verification of these
proofs does not necessitate any `ProofSpecs`.

Next, we discuss the underlying types/fields within the `ConsensusState` struct.

### SovereignConsensusParams

In Sovereign SDK, as previously mentioned, there isn't a direct consensus
concept akin to traditional chains. The rollup obtains its timestamp from the DA
header and synchronizes with it. Consequently, the only parameter that makes
sense to keep rollup-wide as a consensus parameter is the root hash of the
rollup state. During client updates, this root hash doesn't participate in
validations, but it is stored for later use when the light client needs to
validate incoming JMT commitment proofs against the root.

As mentioned, Sovereign SDK employs the Jellyfish Merkle Tree for storing
states, where the data is maintained as a flat store, mapping `(Key, Version)`
tuples to values. Here is the overview of the rollup-wide consensus struct:

```rust
pub struct SovereignConsensusParams {
    pub root: CommitmentRoot,
}
```

This root is obtained during client updates from the `final_state_root` field of
the `AggregatedProofPublicData` struct within an aggregated proof.

### TendermintConsensusParams

Under the `TendermintConsensusParams` we have parameters similar to Tendermint
`ConsensusState` expect the root as described above, which should come from the
execution layer. Particularly we care about the `next_validators_hash` by
capturing the hash of the upcoming validators set to detect potential DA forks.
It ensures smooth DA consensus transitions and validates the Celestia core
header trustworthiness. Additionally, the `timestamp` field plays a crucial role
in ensuring header monotonicity and assisting in packet timeouts.

```rust
pub struct TendermintConsensusParams {
    pub timestamp: Time,
    pub next_validators_hash: Hash,
}
```

## Light Client Implementation

Following defining the factors necessary to operate Sovereign light clients, now
it is time to go over each of the ibc-rs APIs and see how we implement methods
under the
[`ClientState`](https://github.com/cosmos/ibc-rs/blob/2b9de3413eb34179dc0266e851a1c4d4715c8cb8/ibc-core/ics02-client/context/src/client_state.rs#L210-L223)
traits. In cases where the implementation has the same logic as Tendermint light
clients, we denote that with "Same as ICS-07" and just explain where differences
exist.

### ClientStateCommon methods

- `verify_consensus_state()`: Verifies if the incoming `Any` consensus state is
  correctly decoded to the expected domain type and ensures that the root is not
  empty. (Same as ICS-07)
- `client_type()`: Constructs the `ClientType` using the prefix
  `100-sov-celestia`. The name choice aligns with naming conventions across
  different light clients. Though, the number prefix is an arbitrary value
  adjustable in the future.
- `latest_height()`: Returns the `latest_height` from the `sovereign_params` of
  the `ClientState`.
- `validate_proof_height()`: Checks that the height of a receiving commitment
  proof is less than the client's latest height (Same as ICS-07)
- [`verify_upgrade_client()`](https://github.com/informalsystems/sovereign-ibc/blob/a9aaa80c4fe7b21fa777ae2a186838aac1fed68c/clients/sov-celestia/src/client_state/common.rs#L100): Implements the upgrade client verification logic
  (Same as ICS-07)
- [`verify_(non)membership()`](https://github.com/informalsystems/sovereign-ibc/blob/a9aaa80c4fe7b21fa777ae2a186838aac1fed68c/clients/sov-celestia/src/client_state/common.rs#L155):
  Verifies the (non)existence of a JMT commitment in the state tree. The logic
  is as follows:
  - Converts the root bytes into a `[u8; 32]` array as required by the
    `verify_existence()` method of the `jmt` crate.
  - Calculates the JMT key hash using the specified prefix and path. The prefix
    determination is nuanced in the Sovereign SDK system, depending on the path
    type. The commitment prefix appears as `sov_ibc/Ibc`. However, for packet
    commitments, we require the full prefix, such as
    `sov_ibc/Ibc/packet_commitment_map`, as an example. The last section
    corresponds to the name of that state field under the `sov-ibc` module.
  - Obtains the `jmt::proof::SparseMerkleProof` type from the proof bytes.
  - Calls `verify_(non)existence` on the obtained proof type.
  - Here is how it is implemented:

    ```rust
    pub fn verify_membership(
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: Path,
        value: Vec<u8>,
    ) -> Result<(), ClientError> {
        let root_bytes: [u8; 32] = root.as_bytes().try_into().map_err(|_| ClientError::Other {
            description: "invalid commitment root, expected 32 bytes".into(),
        })?;

        let key_hash = obtain_key_hash(prefix, path)?;

        let proof = SparseMerkleProof::<sha2::Sha256>::try_from_slice(proof.as_ref()).map_err(|e| {
            ClientError::InvalidCommitmentProof(CommitmentError::DecodingFailure(e.to_string()))
        })?;

        proof
            .verify_existence(root_bytes.into(), key_hash, value)
            .map_err(|e| ClientError::Other {
                description: e.to_string(),
            })?;

        Ok(())
    }
    ```

  - For the `obtain_key_hash()`, the algorithm is as follows:

    ```rust
    fn obtain_key_hash(prefix: &CommitmentPrefix, path: Path) -> Result<jmt::KeyHash, ClientError> {
        let (prefix_map, encoded_key) = match path {
            Path::ClientState(p) => ("client_state_map", p.try_to_vec()),
            Path::ClientConsensusState(p) => ("consensus_state_map", p.try_to_vec()),
            Path::Connection(p) => ("connection_end_map", p.try_to_vec()),
            Path::ChannelEnd(p) => ("channel_end_map", p.try_to_vec()),
            Path::SeqSend(p) => ("send_sequence_map", p.try_to_vec()),
            Path::SeqRecv(p) => ("recv_sequence_map", p.try_to_vec()),
            Path::SeqAck(p) => ("ack_sequence_map", p.try_to_vec()),
            Path::Commitment(p) => ("packet_commitment_map", p.try_to_vec()),
            Path::Ack(p) => ("packet_ack_map", p.try_to_vec()),
            Path::Receipt(p) => ("packet_receipt_map", p.try_to_vec()),
            Path::UpgradeClient(p) => match p {
                UpgradeClientPath::UpgradedClientState(_) => {
                    ("upgraded_client_state_map", p.try_to_vec())
                }
                UpgradeClientPath::UpgradedClientConsensusState(_) => {
                    ("upgraded_consensus_state_map", p.try_to_vec())
                }
            },
            _ => Err(ClientError::Other {
                description: "unsupported path".into(),
            })?,
        };

        let encoded_key = encoded_key.map_err(|_| ClientError::Other {
            description: "failed to encode key".into(),
        })?;

        let key_bytes = compute_key_bytes(prefix, prefix_map, encoded_key);

        Ok(jmt::KeyHash::with::<sha2::Sha256>(key_bytes.as_slice()))
    }
    ```

### ClientStateValidation methods

- [`verify_client_message()`](https://github.com/informalsystems/sovereign-ibc/blob/09a818d57fed253a500f731eec93c4945df243ad/clients/sov-celestia/src/client_state/validation.rs#L64):
  Contains the verification steps akin to ICS-07 for the `da_header` field
  contained in the [`Header`](#header) struct. The brief overview of the primary
  steps involved in the `SovTmHeader` verification are as follows:

  ```rust
  /// Verifies the IBC header type for the Sovereign SDK rollups, which consists
  /// of the DA header and the aggregated proof date validation.
  pub fn verify_header<V, H>(
      ctx: &V,
      client_state: &SovTmClientState,
      header: &SovTmHeader,
      client_id: &ClientId,
      verifier: &impl TmVerifier,
  ) -> Result<(), ClientError>
  where
      V: ExtClientValidationContext,
      V::ConsensusStateRef: Convertible<SovTmConsensusState, ClientError>,
      H: MerkleHash + Sha256 + Default,
  {
      // Checks the sanity of the fields in the header.
      header.validate_basic::<H>()?;

      header.validate_da_height_offset(
          client_state.genesis_da_height(),
          client_state.latest_height_in_sov(),
      )?;

      verify_da_header::<V, H>(ctx, client_state, &header.da_header, client_id, verifier)?;

      verify_aggregated_proof(
          ctx,
          client_state.genesis_state_root(),
          client_state.code_commitment(),
          &header.aggregated_proof,
      )?;

      Ok(())
  }
  ```

  - This notably includes a check for the occurrence of DA height offsets. This
    consists of the confirming the presence of offsets for both the target and
    trusted heights as follows:

  ```rust
   pub fn validate_da_height_offset(
        &self,
        genesis_da_height: Height,
        client_latest_height: Height,
    ) -> Result<(), ClientError> {
        let expected_da_height = self.height().add(genesis_da_height.revision_height());

        let given_da_height = self.da_header.height();

        if expected_da_height != given_da_height {
            return Err(ClientError::Other {
                description: format!(
                    "The height of the DA header does not match expected height:\
                    got '{given_da_height}', expected '{expected_da_height}'",
                ),
            });
        }

        let client_height_in_da = client_latest_height.add(genesis_da_height.revision_height());

        let header_trusted_height = self.da_header.trusted_height;

        if client_height_in_da != header_trusted_height {
            return Err(ClientError::Other {
                description: format!(
                    "trusted DA height does not match expected height:\
                    got {header_trusted_height}, expected {client_height_in_da}",
                ),
            });
        };

        Ok(())
    }
  ```

  - Additionally, we have included basic validation for the `aggregated_proof`
    such as ensuring that fields are not empty and do not surpass certain sizes.
    Following discussions with the Sovereign team, we have established a limit
    of less than 256 bytes for all fields except the `code_commitment`, which
    can vary in size but can be capped at 10KB.

  - As for the `verify_aggregated_proof()` it already contains a few checks
    ensures that the receiving aggregated proof has the same
    `genesis_state_root` and `code_commitment` as the installed `ClientState`.
    However the core `AggregatedProof` verifier yet to be implemented, but
    assuming the verifier will be imported from `sovereign-sdk`, we should end
    up with a function call like this:

    ```rust
    aggregated_proof.verify()?;
    ```

- [`check_for_misbehaviour()`](https://github.com/informalsystems/sovereign-ibc/blob/09a818d57fed253a500f731eec93c4945df243ad/clients/sov-celestia/src/client_state/validation.rs#L91):
  This already only included the logic for detecting misbehavior in the DA
  layer. Therefore, the logic remains the same as ICS-07.
- [`status()`](https://github.com/informalsystems/sovereign-ibc/blob/09a818d57fed253a500f731eec93c4945df243ad/clients/sov-celestia/src/client_state/validation.rs#L127):
  Returns the status of the client, which has the same logic as ICS-07.
- [`check_substitute()`](https://github.com/informalsystems/sovereign-ibc/blob/09a818d57fed253a500f731eec93c4945df243ad/clients/sov-celestia/src/client_state/validation.rs#L173):
  The logic for the Tendermint-specific parameters remains the same as ICS-07.
  For the rollup-wide fields, we ensure that the `genesis_da_height`,
  `genesis_state_root`, `code_commitment`, and `upgrade_path` match the existing
  client state, as the rollup software supposed to be the same. However, the
  `trusting_period`, `frozen_height`, and `latest_height` are allowed to be
  updated.

### ClientStateExecution methods

- [`initialise()`](https://github.com/informalsystems/sovereign-ibc/blob/09a818d57fed253a500f731eec93c4945df243ad/clients/sov-celestia/src/client_state/execution.rs#L88):
  Initializes the client state, which has the same logic as ICS-07.
- [`update_state()`](https://github.com/informalsystems/sovereign-ibc/blob/09a818d57fed253a500f731eec93c4945df243ad/clients/sov-celestia/src/client_state/execution.rs#L128):
  Updates the client state, which has the same logic as ICS-07 and includes the
  consensus state pruning logic.
- [`update_state_on_misbehaviour()`](https://github.com/informalsystems/sovereign-ibc/blob/09a818d57fed253a500f731eec93c4945df243ad/clients/sov-celestia/src/client_state/execution.rs#L189):
  Updates the client state on misbehavior, which has the same logic as ICS-07.
- [`update_state_on_upgrade()`](https://github.com/informalsystems/sovereign-ibc/blob/09a818d57fed253a500f731eec93c4945df243ad/clients/sov-celestia/src/client_state/execution.rs#L210):
  The logic for the Tendermint-specific parameters remains the same as ICS-07.
  All chain-chosen parameters come from committed (upgraded) client, all
  relayer-chosen parameters come from current client. For the rollup-wide
  fields, the `genesis_state_root` are considered immutable properties of the
  client. Changing them implies creating a client that could potentially be
  compatible with other rollups. Though the `code_commitment` can be updated
  which implies that the rollup software has been updated.
- [`update_on_recovery()`](https://github.com/informalsystems/sovereign-ibc/blob/09a818d57fed253a500f731eec93c4945df243ad/clients/sov-celestia/src/client_state/execution.rs#L297):
  Permitted parameters as specified in the `check_substitute()`, comes from the
  substitute client. Overall, the logic is the same as ICS-07.

It is worth noting that some implementations shared with ICS-07 are imported
from `ibc-rs`. While many functions here could potentially also come from there,
the current state of `ibc-rs` does not offer them as generic over the client
type. Enhancements in `ibc-rs` could streamline shared implementations and avoid
redundancy.

## CosmWasm Contract Implementation

The implementation of the CosmWasm contract relies on types, bindings and a
`Context` object that connects an `ibc-rs` light client to the Cosmwasm storage.
This implementation has undergone several iterations to ensure that the
`Context` is generic enough to accommodate various client types. Consequently,
all abstractions related to this `Context` and the implemented traits have been
moved to `ibc-rs`, living as a standalone
[`ibc-client-cw`](https://github.com/cosmos/ibc-rs/tree/main/ibc-clients/cw-context)
library.

Within the `sovereign-ibc` repository, we maintain the CosmWasm contract
implementation for the `sov-celestia` in the `sov-celestia-client-cw` crate.
This implementation imports all required artifacts from `ibc-client-cw` and
primarily consists of two parts: (1) integrating the client type into the
`Context`, and (2) assembling the `Context` within the contract's entry points.

Here is a brief overview of the contract implementation:

```rust
use ibc_core::derive::ConsensusState as ConsensusStateDerive;
use ibc_client_cw::api::ClientType;
use ibc_client_cw::context::Context;
use sov_celestia_client::client_state::ClientState;
use sov_celestia_client::consensus_state::ConsensusState;

pub struct SovTmClient;

impl<'a> ClientType<'a> for SovTmClient {
    type ClientState = ClientState;
    type ConsensusState = AnyConsensusState;
}

#[derive(Clone, Debug, ConsensusStateDerive)]
pub enum AnyConsensusState {
    Sovereign(ConsensusState),
}

pub type SovTmContext<'a> = Context<'a, SovTmClient>;

#[entry_point]
pub fn instantiate(
    deps: DepsMut<'_>,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let mut ctx = SovTmContext::new_mut(deps, env)?;

    let data = ctx.instantiate(msg)?;

    Ok(Response::default().set_data(data))
}

#[entry_point]
pub fn sudo(deps: DepsMut<'_>, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    let mut ctx = SovTmContext::new_mut(deps, env)?;

    let data = ctx.sudo(msg)?;

    Ok(Response::default().set_data(data))
}

#[entry_point]
pub fn query(deps: Deps<'_>, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let ctx = SovTmContext::new_ref(deps, env)?;

    ctx.query(msg)
        .map_err(|e| StdError::generic_err(e.to_string()))
}
```

To obtain the Wasm file, we compile the contract using the following command:

```bash
make build-sov-celestia-cw
```

And to optimize the Wasm files, we run:

```bash
make optimize-contracts
```

## References

To learn more about the light client implementation and relevant issues / PRs,
please refer to the following tracking issue:

- Keep track of Sovereign light client implementation
  [#2](https://github.com/informalsystems/sovereign-ibc/issues/2)
