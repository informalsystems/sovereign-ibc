# Navigate Workflow and RPC Calls in Relayer Operations

This document provides a high-level picture of how the Hermes relayer works. It
explains the step-by-step process and helps to understand when relayers make RPC
calls to each of the Rollup or Cosmos nodes. Generally, three fundamental tasks
are handled by the relayer’s binary:

1. **Bootstrap Chain Handlers:** Initializes chain handlers and sets the runtime
and configurations for subsequent processes.

2. **Payload Construction:** Obtains necessary data and events from chain nodes
and construct IBC payload for transmission between nodes.

3. **Payload Submission:** Once the payload is ready, the relayer dispatches it,
ensuring it reaches the destination and collects result events for follow-up
actions.

We first go through the workflow involved in (1) and (3), as these consistently
play pivotal roles during the operation of the relayer. Afterward, we explore
step (2) for each of the IBC message types and outline the process of crafting
their payloads. Any step that requires an RPC call to the Sovereign rollup node
is denoted by [S] and to the Cosmos node by [C].

## Table of Contents

- [Navigate Workflow and RPC Calls in Relayer Operations](#navigate-workflow-and-rpc-calls-in-relayer-operations)
  - [Table of Contents](#table-of-contents)
  - [Bootstrap chain handlers](#bootstrap-chain-handlers)
  - [Payload submission](#payload-submission)
  - [Payload construction](#payload-construction)
    - [Create Client for SOV on COS](#create-client-for-sov-on-cos)
    - [Update Client of SOV on COS](#update-client-of-sov-on-cos)
    - [Receive Packet on COS from SOV](#receive-packet-on-cos-from-sov)

## Bootstrap chain handlers

1. Loads configuration settings from the `config.toml` file
   ([See](https://github.com/informalsystems/hermes/blob/main/config.toml) for
   example)
2. Spawns runtimes for both the source and destination chains.
3. Establishes HTTP client connection
4. [S] Checks the status of the rollup nodes through either `sequencer_health`
   or preferably an endpoint like `ledger_rollupStatus` that would return any
   other node info relayer should be aware of too.
5. Checks the status of Celestia node by doing the same RPC calls we do for
       Tendermint/Comet.
6. Initializes key store and load keys by reading params such as key name,
   address type, account prefix, etc., from the config file.
7. Bootstraps handler objects for each chain, encompassing addresses,
   configurations, keyrings, and RPC clients for interaction with the nodes.
   (See
   [`CosmosSdkChain`](https://github.com/informalsystems/hermes/blob/afc46a752c5a1a366e15df87def2a875167d97c4/crates/relayer/src/chain/cosmos.rs#L147-L161)
   for example)

## Payload submission

Once the IBC payloads are constructed and prepared (see next section), the
relayer follows the below steps for broadcasting the payload and obtaining the
result.

1. [S] Retrieves account information essential for signing and encoding a
   transaction
   - Through `accounts_getAccount`
2. [S] Estimates transaction fees through a transaction simulation
   - For the 1st phase, we utilize a predefined fee.
3. Signs and encodes the transaction.
4. [S] Submits the transaction and gets its hash.
   - This is done through `sequencer_publishBatch`
5. [S] Monitors the result by actively waiting for the commitment on the rollup
   - Whether via polling `ledger_getHead` or notifications from
       `ledger_subscribeSlots`

## Payload construction

In this section, we explore how the relayer interacts with each node and
assembles various data to craft IBC payloads for submission. In our explanation,
Sovereign rollups are denoted as SOV, and Cosmos chains as COS for simplicity.

### Create Client for SOV on COS

1. Retrieves the COS signer's account and reads the key from the seed file.
2. [S] Queries the latest committed height (slot number) of SOV
    - Through `prover_AggregatedProofData` with no params to get the most
       recent one.
    - The root of trust for the light client is at this height.
3. [S] Obtains client state settings
    - Retrieves params like `trusting_period`, and consider any optional user
       overrides.
    - Required because light client verifiers use this parameter to verify the
       update client messages
4. Constructs a `ClientState` object for SOV.
5. [S] Constructs a `ConsensusState`  for SOV at the latest height.
    - Obtain the latest root hash from the earlier `prover_AggregatedProofData`
       call.
    - Fetches the DA’s header, verify it and extract the `timestamp` and
       `next_validator_hash` values.
6. Assembles a `MsgCreateClient` payload to be sent to COS.

- NOTE: The code commitment needs to be deployed beforehand on a specified path
  in-accessible to the Wasm light client. Relayer would obtain that with
 `prover_codeCommitment`

### Update Client of SOV on COS

1. [C] Queries the consensus state on COS at the target height.
    1. skip the update if the consensus state already exists at the target
       height
2. [C] Gets the latest SOV client state on COS to validate it
    1. client is not frozen
    2. client has not “expired”: the latest trusted consensus state on COS is
       not outside the trusting period. For this we need to:
        1. get the timestamp of the DA header at the height of the trusted
           consensus state height
        2. get the timestamp of the DA header at the height of the target
           height. Typically this is the “latest” which means the timestamp of
           the latest block that includes an aggregate proof
        3. make sure they are within the trusting period
3. [S] Queries the latest committed height (slot number) of SOV
    - Through `prover_aggregatedProofData` with no params to get the most
       recent one.
    - The updated state of the light client will be at this **target** height
4. [S] Fetches the DA's header at **target** height
    - We assume here we can do the same RPC calls on Celestia that we do for
       Tendermint/Comet
5. Verifies the DA header (and maybe the aggregate proof)
6. Creates the overall Sovereign header (DA header + proof) and creates the
   `MsgUpdateClient`
7. Retrieves the COS signer's account and reads the key from the seed file.
8. Creates a transaction, simulate, sign and broadcast (see Payload Submission)

### Receive Packet on COS from SOV

**On start:**

1. [S] Queries all pending packet sequences on SOV through
   `ibc_packetCommitments` at latest aggregate proof height.
    1. Needs to know the height of the latest aggregate proof
    2. Need the ability to provide the height context to queries in general
2. [C] Queries on COS for all unreceived packets out of the pending sequences
   obtained above with `ibc_unreceivedPackets`
3. [S] Retrieves the packet data for the sequences above via
   `ledger_getEventsRange`
    1. `sov-ibc` emits packet_key event along with the regular send_packet event
    2. hermes has the `packet_key` for each packets from step 1
    3. hermes queries the ledger events with the `packet_key`
4. Builds `MsgRecvPacket` message for each pending packet that has not timed out
    - This requires the jelly fish proof query for the packet commitment with
       `ibc_packetCommitment`
5. Builds `timeout` messages for packets that have expired, which will be sent
   to SOV
6. Constructs a `MsgUpdateClient` message with the aggregated proof and the DA
   header (as covered by the Update Client section)
7. Prepare a batch of update client and received packet messages to be sent to
   COS

**Normal operation in pull mode:**

In this mode the relayer queries the chains periodically. In current
implementation the frequency is specified through a duration configuration. For
SOV could be the same or triggered by the aggregate proof notification presence
`prover_subscribeAggregatedProof`

In all cases the relayer saves the last processed height as `start_height`

When the pull trigger happens:

1. [S] Collects emitted `SendPacket` events from SOV from `start_height` and up
   to the `latest_height` (slot number) for which `AggregatedProofData` exists.
    1. Like in the case of start, the relayer queries all pending commitments
       with `ibc_packetCommitments` on SOV
    2. [C] It then finds the sequences of unreceived packets on COS
    3. Gets the packet data as covered in previous section
2. continue as above

**Other packet types:**

The `MsgTimeout`, `MsgAcknowledgment` and all the other packet messages have the
same requirements as for the received packets.

Also the channel and connection handshake messages have similar requirements.
One peculiar one:

- the connection handler on chain A must have access to “recent” headers of
  chain A
- this is because the counterparty chain (B) must prove that its client for A
  has a “recent” consensus state.
- in cosmos “recent” is defined in number of heights via a IBC genesis/
  governance parameter `historical-entries`
