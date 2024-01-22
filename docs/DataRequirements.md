# Hermes IBC Relayer Data Availability Requirements

## Changelog

- 2024-01-17: Drafted initial requirements
- 2024-01-18: Applied review feedback

## Context

The following endpoints (or equivalent) are necessary for operating the relayer.
An optimal approach involves exposing these endpoints as methods on the unified
client designed to manage requests and responses by various RPC or WebSocket
connections. For each section, we provide a comprehensive list of the endpoints,
**their priority for the initial phase of implementation** and latest
availability status, as far as we could investigate in the Sovereign SDK
codebase. They are ordered from highest to lowest impact roughly, i.e., the last
endpoint in the list is the least important and least frequently required.

## Table of Contents

- [Hermes IBC Relayer Data Availability Requirements](#hermes-ibc-relayer-data-availability-requirements)
  - [Changelog](#changelog)
  - [Context](#context)
  - [Table of Contents](#table-of-contents)
  - [Sequencer RPC](#sequencer-rpc)
    - [`/sequencer_submitTxs`](#sequencer_submittxs)
    - [`sequencer_txStatus`](#sequencer_txstatus)
    - [`/sequencer_health`](#sequencer_health)
  - [Rollup RPC](#rollup-rpc)
    - [`/ledger_searchTx`](#ledger_searchtx)
    - [`/prover_aggregatedProof*`](#prover_aggregatedproof)
    - [`/ledger_rollupParams`](#ledger_rollupparams)
    - [`/ledger_rollupStatus`](#ledger_rollupstatus)
    - [`/rollup_health`](#rollup_health)
    - [IBC modules RPC](#ibc-modules-rpc)
      - [Channel endpoints](#channel-endpoints)
      - [Client endpoints](#client-endpoints)
      - [Connection endpoints](#connection-endpoints)
  - [Rollup WebSocket](#rollup-websocket)
    - [`/ledger_subscribeAggregatedProof`](#ledger_subscribeaggregatedproof)
    - [`/ledger_subscribeSlots`](#ledger_subscribeslots)

## Sequencer RPC

### `/sequencer_submitTxs`

- Objective:
  - For submitting batch of transactions into the mempool.
  - To simulate transaction sending and conduct basic pre-send
    checks on factors like transaction size and gas fees, etc.

- Priority: High

- Status:
  - Nothing available to handle (accept and submit) a batch of transactions in
    one go.
  - There is a [`sequencer_acceptTx`](https://github.com/Sovereign-Labs/sovereign-sdk/blob/190863c29835af9090e38d79284b24406c33758c/full-node/sov-sequencer/src/lib.rs#L56-L64) method, which stores a transaction into the mempool.
  - Also
  [`sequencer_publishBatch`](https://github.com/Sovereign-Labs/sovereign-sdk/blob/cca1729445741aadbec2490c14ca2090afdc878b/full-node/sov-sequencer/src/lib.rs#L74-L90)
  method for submitting batch of transactions to the DA layer which works in an
  async fashion.

### `sequencer_txStatus`

- Objective:
  - Used to check the submission and commitment status of pending transactions
    on the DA layer, so can decide on transaction re-submission if necessary.

- Priority: Low

- Status: Nothing available yet.

### `/sequencer_health`

- Objective:
  - Needed for basic check to assess the health of sequencer node.
  - Only used once, at relayer startup during health check.
  
- Priority: Low

- Status: Available as the
  [`/health`](https://github.com/Sovereign-Labs/sovereign-sdk/blob/1adbfc963bb930edfa0efe6030262dfb70acf199/module-system/sov-modules-macros/src/rpc/rpc_gen.rs#L339-L343)
  method to check the health of the RPC server.

## Rollup RPC

### `/ledger_searchTx`

- Objective:
  1. To obtain packet events that occurred during a range of heights at or
     before a specified height. Required because rollup state does not store the
     full packet data which is needed to build and relay the packet messages.
     - Pattern:
       - Used relatively often, on start and then for every `z` blocks, where
         `clear_interval = z` (default `z = 100`).
       - `send_packet.packet_src_channel == X &&
       send_packet.packet_src_port == X && send_packet.packet_dst_channel == X
       && send_packet.packet_dst_port == X && send_packet.packet_sequence ==
       X`. Also for `write_acknowledgement` packet events.
       - `height > initial_state_height && height <= final_state_height`
     - Priority: High

  2. To obtain client update events: (a) for the misbehavior detection task, and
     (b) for relaying packets on connections that have non-zero delay. (Not
     priority)
     - Used rarely in practice because all connections have 0 delay and often
       misbehavior detection is disabled.
     - Pattern:
       - `update_client.client_id == client_id && update_client.consensus_height == X-Y`
       - `height > initial_state_height && height <= final_state_height`
     - Priority: Low

  3. To obtain transaction events, for confirming if packets are committed to
     the rollup.
     - Not needed on the critical path of packet relaying. Used very often as
       part of packet confirmation.
     - Pattern: `tx.hash == XYZ`
     - Priority: Nice to have

  4. For the success/error status of a transaction immediately after it was
     broadcast.
     - Used rarely at bootstrap (to register counterparty payee address for
       fees) or when transactions need to be sent sequentially.
     - Used on all transactions when `tx_confirmation` config is enabled. (At
       initial phase would be set to false)
     - Pattern: `tx.hash == XYZ`
     - Priority: Nice to have

- Status:
  - Regarding the 1st and 2nd situations, nothing straightforward available yet
    to search for all the events with particular key, where events might have been
    emitted by the same transaction.
  - The `ledger_getTransactions` RPC method enables search for txs using the
    hash. This method returns a list of all the events emitted by that tx.
  - There is also a `ledger_getTransactionsRange` method. Each transaction has a
    monotonically increasing ID. So this could be used as a range query if we
    know the ID of the start or end transaction.
  - The `ledger_getEvents` RPC method enables search for a single event using
    the provided
    [`EventIdentifier`](https://github.com/Sovereign-Labs/sovereign-sdk/blob/main/rollup-interface/src/node/rpc/mod.rs#L80-L92),
    which can be a transaction ID but not a transaction hash.

- Remark:
  - Ideally the endpoint should support a query language, enabling the inclusion
    of ANDed conditions to facilitate event searches.

### `/prover_aggregatedProof*`

- Objective:
  - To obtain an aggregated proof and its relevant data. The relayer must
    operate on rollup data that has been proven. These methods may be exposed
    either by the DA layer or the rollup node, including:

  - `/prover_aggregatedProofData`: Used to construct the IBC header for passing
    into the rollup clients, so they can verify aggregated proof and update their
    client state. By default, it returns the latest published aggregated proof,
    but we should be able to query for a specific height by passing e.g.
    `proofIdentifier`.

  - `/prover_latestProofDataInfo`: Used as a cheaper convenient endpoint for
    catching up the relayer to the latest executed DA block and for status checks.

- Priority: High

- Status: Nothing available yet.

- Remark:
  - When clearing packets for a height range beyond a single proof's coverage,
    it is unclear whether we can rely solely on the latest proof for
    constructing an update client message and send it along with the rest of
    packets to the counterparty chain.

### `/ledger_rollupParams`

- Objective:
  - Used for adjusting rollup parameters that the relayer may need for setting
    configurations and basic checks, like the namespaces, max batch size or max
    tx size, max gas fees, etc.
  - Used to retrieve the rollup code commitment, essential for the aggregated
    proof verification.
  - Usually used once, at relayer startup during health check.
  - Usually not needed for IBC relaying strictly speaking.

- Priority: Low

- Status: Nothing available yet.

- Remark:
  - The exact parameters that the relayer needs to retrieve from the rollup
    node should be determined.
  - The frequency of updating the code commitment and where this update should
    happen is unclear.

### `/ledger_rollupStatus`

- Objective:
  - Needed to get rollup status including node info, latest DA block hash,
    rollup state root, height (slot number) and time.
  - Assuming the `/status` returns similar
    [`Response`](https://github.com/informalsystems/tendermint-rs/blob/main/rpc/src/endpoint/status.rs#L26-L37)
    type to Cosmos chains, The response used in two situations:
    1. At relayer startup to fetch `node_info.id`, for initializing the light
       client component and verify the relayer Chain ID is the same as the full
       node network, so ensure we connect to the right chain or/and rollup.
    2. To fetch the rollup latest time and height used in many methods, often
      alongside `node_info.network`, for example:
        - As a dependency, because the latest height is necessary in calling the
          `/ledger_rollupParams` RPC, during health check.
        - Needed in channel handshake (open try, ack, confirm; close confirm).
        - In connection handshake (open try, ack, confirm), for both the source
          chain (to update client), and destination chain (to construct proofs).
        - Note: It seems like we bombard the node with `/*_status` queries, but most
          of the queries hit the Hermes in-process cache.
        - For updating clients during everyday IBC relaying. In case there is
          non-zero connection delay, we again bombard the node with `/*_status`
          queries.

- Priority: Low

- Status: Nothing available yet, but part of the needed data can be gathered
  from different endpoints like `ledger_getHead` or
  `prover_AggregatedProofData`.

### `/rollup_health`

- Objective:
  - Needed for basic check to assess the health of the rollup nodes.
  - Only used once, at relayer startup during health check.
  - Not needed for IBC relaying strictly speaking.
  - In case of Cosmos chains, it returns empty result (200 OK) on success.

- Priority: Low

- Status: Available as a
  [`/health`](https://github.com/Sovereign-Labs/sovereign-sdk/blob/1adbfc963bb930edfa0efe6030262dfb70acf199/module-system/sov-modules-macros/src/rpc/rpc_gen.rs#L339-L343)
  method to check the health of the RPC server.

### IBC modules RPC

- Objective:
  - Queries client, connection and channel-associated data and states.
  - Used to retrieve commitment proofs for every IBC message relayed
  - To obtain the client upgraded state, while the relayer is handling chain
    upgrades.

- Priority: High

- Status: RPC methods Implemented expect for the upgraded client state query.

#### Channel endpoints

- `ibc_channelClientState`: Requests the client state associated with a
  specified channel.
- `ibc_channels`: Requests all of the channels associated with the chain.
- `ibc_channelConnections`: Requests the connection associated with a specified
  channel.
- `ibc_nextSequenceReceive`: Requests the sequence number of the next receive
  packet for a specified channel.
- `ibc_packetCommitments`: Requests the packet commitments associated with a
  specified channel.
- `ibc_packetAcknowledgements`: Requests the packet acknowledgments associated
  with a specified channel. -`ibc_unreceivedAcks`: Requests the unreceived
acknowledgments associated with a specified channel. -`ibc_unreceivedPackets`:
Requests the unreceived packet sequences associated with a specified channel.

> _*Note:*_ The `PacketAcknowledgements`, `UnreceivedAcks`, and
> `UnreceivedPackets` queries each accept a vector of `sequences` in order to
> specify which packet commitments to fetch acknowledgements for. In the case
> where an empty vector is passed, the queries will simply return all
> acknowledgements for all outstanding packet commitments.

#### Client endpoints

- `ibc_clientStates`: Requests all client states associated with the chain.
- `ibc_ConsensusStates`: Requests all the consensus states associated with a
  specified client.
- `ibc_consensusStateHeights`: Requests all the consensus state heights
  associated with a specified client.
- `ibc_upgradedClientState`: Requests the upgraded client state associated with a
  specified client.

#### Connection endpoints

- `ibc_clientConnections`: Requests all connections associated with a specified client.
- ‍`ibc_connections`: Requests all connections associated with the chain.

## Rollup WebSocket

### `/ledger_subscribeAggregatedProof`

- Objective:
  - Subscribe to the rollup node's websocket and listen to aggregated proofs every
    time a proof is generated and committed on the DA layer.
  - Can obtain the height of latest committed DA block (slot number) from the
    aggregated proof data.

- Priority: Low

- Status: Nothing available yet.

### `/ledger_subscribeSlots`

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
