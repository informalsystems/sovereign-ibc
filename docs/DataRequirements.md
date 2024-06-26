# Hermes IBC Relayer Data Availability Requirements

## Changelog

- 2024-01-17: Drafted initial requirements
- 2024-01-18: Applied review feedback
- 2024-03-25: Updated with changes in Sovereign SDK
- 2024-04-07: Updated with changes in Sovereign SDK

## Context

The following endpoints (or equivalent) are necessary for operating the relayer.
An optimal approach involves exposing these endpoints as methods on the unified
client designed to manage requests and responses by various RPC or WebSocket
connections. For each section, we provide a comprehensive list of the endpoints,
**their priority for the initial phase of implementation** and the latest
availability status, as far as we could investigate in the Sovereign SDK
codebase. They are ordered from highest to lowest impact roughly, i.e., the last
endpoint in the list is the least important and least frequently required.

## Table of Contents

- [Hermes IBC Relayer Data Availability Requirements](#hermes-ibc-relayer-data-availability-requirements)
  - [Changelog](#changelog)
  - [Context](#context)
  - [Table of Contents](#table-of-contents)
  - [Sequencer RPC](#sequencer-rpc)
    - [`sequencer_publishBatch`](#sequencer_publishbatch)
    - [`sequencer_txStatus`](#sequencer_txstatus)
    - [`sequencer_health`](#sequencer_health)
  - [Rollup RPC](#rollup-rpc)
    - [`ledger_getEventsByKey`](#ledger_geteventsbykey)
    - [`ledger_getTransactions`](#ledger_gettransactions)
    - [`ledger_getAggregatedProof*`](#ledger_getaggregatedproof)
    - [`ledger_codeCommitment`](#ledger_codecommitment)
    - [`accounts_getAccount`](#accounts_getaccount)
    - [`ledger_rollupStatus`](#ledger_rollupstatus)
    - [`<namespace>_health`](#namespace_health)
    - [IBC modules](#ibc-modules)
      - [Channel endpoints](#channel-endpoints)
      - [Client endpoints](#client-endpoints)
      - [Connection endpoints](#connection-endpoints)
  - [Rollup WebSocket](#rollup-websocket)
    - [`ledger_subscribeAggregatedProof`](#ledger_subscribeaggregatedproof)
    - [`ledger_subscribeSlots`](#ledger_subscribeslots)

## Sequencer RPC

### `sequencer_publishBatch`

- Objective:
  - For submitting batch of transactions into the mempool.
  - To simulate transaction sending and conduct basic pre-send
    checks on factors like transaction size and gas fees, etc.

- Priority: High

- Status:
  - The
    [`sequencer_publishBatch`](https://github.com/informalsystems/sovereign-sdk-wip/blob/cfdec9397a7d8cb8c64d901b8b573b2160c7a3ed/full-node/sov-sequencer/src/lib.rs#L140-L155)
    method works for this purpose. It takes an optional parameter where we can
    put a list of transactions, and then it will (1) insert all the transactions
    into the mempool, and (2) trigger the mempool to create a batch and post it
    on the DA layer.
  - There is also a
    [`sequencer_acceptTx`](https://github.com/informalsystems/sovereign-sdk-wip/blob/cfdec9397a7d8cb8c64d901b8b573b2160c7a3ed/full-node/sov-sequencer/src/lib.rs#L156-L167)
    method, which stores a single transaction into the mempool.

### `sequencer_txStatus`

- Objective:
  - Used to check the submission and commitment status of pending transactions
    on the DA layer, so can decide on transaction re-submission if necessary.

- Priority: Low

- Status: There is a
  [socket](https://github.com/informalsystems/sovereign-sdk-wip/blob/cfdec9397a7d8cb8c64d901b8b573b2160c7a3ed/full-node/sov-sequencer/src/lib.rs#L178-L185)
  endpoint and [RPC
  method](https://github.com/informalsystems/sovereign-sdk-wip/blob/cfdec9397a7d8cb8c64d901b8b573b2160c7a3ed/full-node/sov-sequencer/src/lib.rs#L169-L177)
  available.

### `sequencer_health`

- Objective:
  - Needed for basic check to assess the health of sequencer node.
  - Only used once, at relayer startup during a health check.

- Priority: Low

- Status: Available as the
  [`health`](https://github.com/Sovereign-Labs/sovereign-sdk/blob/1adbfc963bb930edfa0efe6030262dfb70acf199/module-system/sov-modules-macros/src/rpc/rpc_gen.rs#L339-L343)
  method to check the health of the RPC server.

## Rollup RPC

### `ledger_getEventsByKey`

- Objective:
  1. To obtain packet events that occurred during a range of heights at or
     before a specified height. Required because rollup state does not store the
     full packet data that is needed to build and relay the packet messages.
     This endpoint is used to build and relay packet messages, especially for
     indexing pending packets, where we intend to rely on a `packet_key`. This
     `packet_key` will be a commitment hash, derived from the IBC `send_packet`
     or `write_acknowledgement` events.
     - Pattern:
       - Should allow specifying `start_height`, `end_height` and `event_key` as
         params to filter the events out of the range `height > start_height &&
         height <= end_height`
       - Used relatively often, on start and then for every `z` blocks, where
         `clear_interval = z` (default `z = 100`).
     - Priority: High

  2. To obtain client update events: (a) for the misbehavior detection task, and
     (b) for relaying packets on connections that have non-zero delay.
     - Used rarely in practice because all connections have 0 delay and often
       misbehavior detection is disabled.
     - Pattern:
       - `update_client.client_id == client_id && update_client.consensus_height == X-Y`
       - `height > initial_state_height && height <= final_state_height`
     - Priority: Low

- Status:
  - [Available](https://github.com/informalsystems/sovereign-sdk-wip/blob/7d5595990c691e50d506e2b732b02b363d23ae36/full-node/sov-ledger-rpc/src/server.rs#L148-L154)
  - Additionally worth noting there is a `ledger_getEvents` RPC method enabling
    to search for a single event using the provided
    [`EventIdentifier`](https://github.com/Sovereign-Labs/sovereign-sdk/blob/main/rollup-interface/src/node/rpc/mod.rs#L80-L92),
    which can be a transaction ID but not a transaction hash.

- Remark:
  - For a portion of our indexing needs, this endpoint works as an interim
    solution. For now, `sov-ibc` will introduce a few custom-crafted event
    variants, where the key of these newly defined events is a commitment
    hash (`packet_key`) for distinctiveness. But, ideally, the endpoint should
    support a query language, enabling the inclusion of AND-ed conditions to
    facilitate various types of event searches for cases like:
    - `send_packet.packet_src_channel == X && send_packet.packet_src_port == X
       && send_packet.packet_dst_channel == X && send_packet.packet_dst_port ==
       X && send_packet.packet_sequence == X`

### `ledger_getTransactions`

  1. To obtain transaction events, for confirming if packets are committed to
     the rollup.
     - Not needed on the critical path of packet relaying. Used very often as
       part of packet confirmation.
     - Pattern: `tx.hash == XYZ`
     - Priority: Nice to have

  2. For the success/error status of a transaction immediately after it was
     broadcast.
     - Used rarely at bootstrap (to register counterparty payee address for
       fees) or when transactions need to be sent sequentially.
     - Used on all transactions when `tx_confirmation` config is enabled. (At
       initial phase would be set to false)
     - Pattern: `tx.hash == XYZ`
     - Priority: Nice to have

- Status:
  - The `ledger_getTransactions` RPC method enables search for txs using the
    hash. This method returns a list of all the events emitted by that tx.
  - There is also a `ledger_getTransactionsRange` method. Each transaction has a
    monotonically increasing ID. So this could be used as a range query if we
    know the ID of the start or end transaction.

### `ledger_getAggregatedProof*`

- Objective:
  - To obtain an aggregated proof and its relevant data. The relayer must
    operate on rollup data that has been proven. These methods may be exposed
    either by the DA layer or the rollup node, including:

  - `ledger_getAggregatedProof`:
    - Returns a response of `AggregatedProof` type.
    - Used to construct the IBC header for submission into the rollup light
      clients, so they can verify aggregated proof and update their state. By
      default, it returns the latest published aggregated proof, but we should
      be able to query for a specific height by passing e.g. `proofIdentifier`.

  - `ledger_getAggregatedProofInfo`:
    - Returns the information of `ProofDataInfo` type.
    - Used as a cheaper convenient endpoint for catching up the relayer to the
      latest executed DA block and for status checks.

- Priority: High

- Status:
  [Available](https://github.com/informalsystems/sovereign-sdk-wip/blob/7d5595990c691e50d506e2b732b02b363d23ae36/full-node/sov-ledger-rpc/src/server.rs#L183-L203).
  Returns the latest aggregated proof.

- Remark:
  - When clearing packets for a height range beyond a single proof's coverage,
    it is unclear whether we can rely solely on the latest proof for
    constructing an update client message and send it along with the rest of
    packets to the counterparty chain.

### `ledger_codeCommitment`

- Objective:
  - Used to retrieve the rollup code commitment, essential for the aggregated
    proof verification by IBC light clients.
  - Given that the first stab is to go with an on-chain governance proposal for
    storing the `code_commitment` of a rollup on a counterparty chain, relayers
    need a convenient endpoint to obtain the commitment and include that in the
    governance proposal message. This is based on the assumption that the rollup
    node serves as the most reliable source.

- Priority: High

- Status: Nothing available yet

### `accounts_getAccount`

- Objective:
  - Used to retrieve the account information such as the address and the
    sequence number, primarily used for building and signing transactions.

- Priority: High

- Status:
  - There is an
    [`accounts_getAccount`](https://github.com/informalsystems/sovereign-sdk-wip/blob/2bcb00f98dabbc8ce56abfe9babdfe5fb3979b7b/module-system/module-implementations/sov-accounts/src/rpc.rs#L23-L39)
    RPC endpoint which appears to perform the same job as `query_account` on
    Cosmos chains.

### `ledger_rollupStatus`

- Objective:
  - Needed to get rollup status including node info, latest DA block hash,
    rollup state root, height (slot number) and time.
  - Assuming the `status` returns similar
    [`Response`](https://github.com/informalsystems/tendermint-rs/blob/main/rpc/src/endpoint/status.rs#L26-L37)
    type to Cosmos chains, The response used in two situations:
    1. At relayer startup to fetch `node_info.id`, for initializing the light
       client component and verify the relayer Chain ID is the same as the full
       node network, so ensure we connect to the right chain or/and rollup.
    2. To fetch the rollup latest time and height used in many methods, often
      alongside `node_info.network`, for example:
        - As a dependency, because the latest height is necessary in calling the
          `ledger_rollupStatus` RPC, during health check.
        - Needed in channel handshake (open try, ack, confirm; close confirm).
        - In connection handshake (open try, ack, confirm), for both the source
          chain (to update client), and destination chain (to construct proofs).
        - Note: It seems like we bombard the node with `*_status` queries, but most
          of the queries hit the Hermes in-process cache.
        - For updating clients during everyday IBC relaying. In case there is
          non-zero connection delay, we again bombard the node with `*_status`
          queries.

- Priority: Low

- Status: Nothing available yet, but part of the needed data can be gathered
  from different endpoints like `ledger_getHead` or `ledger_getAggregatedProof`.

### `<namespace>_health`

- Objective:
  - Needed for basic check to assess the health of the rollup nodes.
  - Only used once, at relayer startup during health check.
  - Not needed for IBC relaying strictly speaking.
  - In case of Cosmos chains, it returns empty result (200 OK) on success.

- Priority: Low

- Status: Available as a
  [`health`](https://github.com/Sovereign-Labs/sovereign-sdk/blob/1adbfc963bb930edfa0efe6030262dfb70acf199/module-system/sov-modules-macros/src/rpc/rpc_gen.rs#L339-L343)
  method to check the health of the RPC server.

### IBC modules

- Objective:
  - Queries client, connection and channel-associated data and states.
  - Used to retrieve commitment proofs for every IBC message relayed
  - To obtain the client upgraded state, while the relayer is handling chain
    upgrades.

- Priority: High

- Status: RPC methods Implemented expect for the upgraded client state query.

#### Channel endpoints

- `ibc_channelClientState`: Returns the client state associated with a
  specified channel.
- `ibc_channelConsensusState`: Returns the consensus state associated with a
  specified channel.
- `ibc_channels`: Returns all of the channels associated with the chain.
- `ibc_channelConnections`: Returns the connection associated with a specified
  channel.

- `ibc_packetCommitment`: Returns the commitment and proof of existence for a
  single packet on a specified channel and sequence number.
- `ibc_packetReceipt`: Returns the receipt and proof of existence for a single
  packet on a specified channel and sequence number.
- `ibc_packetAcknowledgement`: Returns the acknowledgment and proof of
  existence for a single packet on a specified channel and sequence number.
- `ibc_nextSequenceReceive`: Returns the sequence number of the next receive
  packet for a specified channel.

- `ibc_packetCommitments`: Returns the packet commitments associated with a
  specified channel.
- `ibc_packetAcknowledgements`: Returns the packet acknowledgments associated
  with a specified channel.
- `ibc_unreceivedAcks`: Returns the unreceived acknowledgments associated with
a specified channel.
- `ibc_unreceivedPackets`: Returns the unreceived packet sequences associated
with a specified channel.

> _*Note:*_ The `PacketAcknowledgements`, `UnreceivedAcks`, and
> `UnreceivedPackets` queries each accept a vector of `sequences` in order to
> specify which packet commitments to fetch acknowledgements for. In the case
> where an empty vector is passed, the queries will simply return all
> acknowledgements for all outstanding packet commitments.

#### Client endpoints

- `ibc_clientStates`: Returns all client states associated with the chain.
- `ibc_consensusStates`: Returns all the consensus states associated with a
  specified client.
- `ibc_consensusStateHeights`: Returns all the consensus state heights
  associated with a specified client.
- `ibc_upgradedClientState`: Returns the upgraded client state associated with
  a specified client.
- `ibc_upgradedConsensusState`: Returns the upgraded consensus state associated
  with a specified client.

#### Connection endpoints

- `ibc_clientConnections`: Returns all connections associated with a specified client.
- ‍`ibc_connections`: Returns all connections associated with the chain.
- `ibc_connection`: Returns the connection associated with a specified
  connection identifier.
- `ibc_connectionParams`: Returns the connection parameters associated with a
  specified connection.

## Rollup WebSocket

### `ledger_subscribeAggregatedProof`

- Objective:
  - Subscribe to the rollup node's websocket and listen to aggregated proofs
    every time a proof is generated and committed on the DA layer.
  - Can obtain the height of latest committed DA block (slot number) from the
    aggregated proof data.

- Priority: Low

- Status: [Available](https://github.com/informalsystems/sovereign-sdk-wip/blob/7d5595990c691e50d506e2b732b02b363d23ae36/full-node/sov-ledger-rpc/src/server.rs#L215-L223)

### `ledger_subscribeSlots`

- Objective:
  - Connect to the rollup node's websocket and subscribes to the ibc events.

- Priority: Low

- Status: The `ledger_subscribeSlots` endpoint provides a stream of
  [`SlotResponse`](https://github.com/Sovereign-Labs/sovereign-sdk/blob/bd469c70fc1227a7785fb177a34de21bb6d5eb08/rollup-interface/src/node/rpc/mod.rs#L136-L149)
  type, encompassing the processed batches.

- Remark:
  - Within this endpoint, we gain access to a list of transactions and so on
    related events. However, for obtaining IBC events, it is necessary to
    implement a filter on the `SlotResponse` type to exclude non-IBC batches,
    txs and events.
