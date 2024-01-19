# Hermes IBC Relayer Data Requirements

## Changelog

- 2024-01-17: Initial requirements

## Context

The following endpoints (or equivalent) are necessary for operating the relayer.
An optimal approach involves encapsulating these endpoints as methods on the
unified clients designed to manage RPC or WebSocket requests and responses. For
each section, we provide a list of the endpoints and the latest status of that
endpoint, as far as we could investigate in the Sovereign SDK codebase. They are
ordered from highest to lowest impact roughly, i.e., the last endpoint in the
list is the least important and least frequently required.

## Table of Contents

- [Hermes IBC Relayer Data Requirements](#hermes-ibc-relayer-data-requirements)
  - [Changelog](#changelog)
  - [Context](#context)
  - [Table of Contents](#table-of-contents)
  - [Sequencer RPC](#sequencer-rpc)
    - [`/send_transactions`](#send_transactions)
    - [`/send_evidence`](#send_evidence)
    - [`/sequencer_health`](#sequencer_health)
  - [Rollup RPC](#rollup-rpc)
    - [`/transaction_search`](#transaction_search)
    - [`/aggregated_proof_search`](#aggregated_proof_search)
    - [`/rollup_params`](#rollup_params)
    - [`/status`](#status)
    - [`/rollup_health`](#rollup_health)
  - [IBC modules RPC](#ibc-modules-rpc)
    - [Channel endpoints](#channel-endpoints)
    - [Client endpoints](#client-endpoints)
    - [Connection endpoints](#connection-endpoints)
  - [Rollup WebSocket](#rollup-websocket)
    - [`/subscribe_aggregated_proofs`](#subscribe_aggregated_proofs)
    - [`/subscribe_events`](#subscribe_events)

## Sequencer RPC

### `/send_transactions`

- For submitting batch of transactions into the mempool.
- It can simulate transaction sending and conduct basic pre-send checks on
  factors like transaction size and gas fees, etc.

- Status: available as a method on the sequencer client, and also as
  [`sequencer_publishBatch`](https://github.com/Sovereign-Labs/sovereign-sdk/blob/cca1729445741aadbec2490c14ca2090afdc878b/full-node/sov-sequencer/src/lib.rs#L74-L90)
  RPC method, though both works in an async fashion.

### `/send_evidence`

- Used for submitting evidence of a rollup misbehaving.

- Status: Nothing available yet.

### `/sequencer_health`

- Needed for basic check to assess the health of sequencer node.
- Only used once, at relayer startup during health check.
  
- Status: Available as the `/health` method on RPC client.

## Rollup RPC

### `/transaction_search`

- Used In the four following situations:

1. Query to obtain transaction events, for confirming if packets are committed
   to the rollup.
   - Not needed on the critical path of packet relaying. Used very often as
     part of packet confirmation.
   - Pattern: `tx.hash == XYZ`

2. Query for the success/error status of a transaction immediately after it was
   broadcast.
   - Used rarely: at bootstrap (to register counterparty payee
         address for fees) or when transactions need to be sent sequentially.
   - Pattern: `tx.hash == XYZ`

3. Query to obtain packet events that occurred at or before a specified height.
   Required because rollup state does not store the full packet data which is
   needed to build and relay the packet messages.
   - Pattern: `send_packet.packet_src_channel == X &&
     send_packet.packet_src_port == X && send_packet.packet_dst_channel == X
     && send_packet.packet_dst_port == X && send_packet.packet_sequence ==
     X`. Also for `write_acknowledgement` packet events.
   - Used relatively often, on start and then for every `z` blocks, where
     `clear_interval = z` (default `z = 100`).

4. Query to obtain client update events: (a) for the misbehavior detection task,
   and (b) for relaying packets on connections that have non-zero delay.
   - Used rarely in practice because all connections have 0 delay and often
     misbehavior detection is disabled.
   - Pattern: `update_client.client_id == NUM-rollup-X &&
     update_client.consensus_height == X-Y`

- Status:
  - The `ledger_getTransactions` RPC method enables search for txs using the
    hash. This method returns a list of all the events emitted by that tx.
  - The `ledger_getEvents` RPC method enables search for a single event using
  the provided
  [`EventIdentifier`](https://github.com/Sovereign-Labs/sovereign-sdk/blob/main/rollup-interface/src/node/rpc/mod.rs#L80-L92),
  which can be a transaction ID but not a transaction hash.
  - Regarding the 3rd and 4th situations, nothing straightforward available yet
  to search for all the events with particular key, where events might have been
  emitted by different transactions.

### `/aggregated_proof_search`

Here is a list of the essential RPC methods to acquire aggregated proof and
related data. These methods may be accessed either through the DA layer or the
rollup node:

- `/prover_aggregatedProofData`: Used to construct the IBC header for passing
  into the rollup clients, so they can verify aggregated proof and update their
  client state. By default, it returns the latest published aggregated proof,
  but we should be able to query for a specific height by passing e.g.
  `proofIdentifier`.

- `/prover_latestProofDataInfo`: Used as a cheaper convenient endpoint for
  catching up the relayer to the latest executed DA block and for status checks.

- `/prover_aggregatedProofsData`: Used to create update client messages,
  specifically in situations where updating a client for a range of slots or
  heights surpasses the coverage provided by a single proof, therefore it
  requires the submission of a list of aggregated proofs.

- status: Nothing available yet.

### `/rollup_params`

- Used for adjusting rollup parameters that the relayer may need for setting
  configurations and basic checks, like the namespaces, max batch size, max tx
  size, max gas fees, etc.
- Only used once, at relayer startup during health check.
- Usually not needed for IBC relaying strictly speaking.

- Status: Nothing available yet.

### `/status`

- Needed to get rollup status including node info, pubkey, latest DA block hash,
  rollup state root, height (slot number) and time.
- Assuming the `/status` returns similar
  [`Response`](https://github.com/informalsystems/tendermint-rs/blob/main/rpc/src/endpoint/status.rs#L26-L37)
  type to Cosmos chains, The response used in two situations:
  1. At relayer startup to fetch `node_info.id`, for initializing the light
     client component.
      - Also at startup to fetch `node_info.other.tx_index`, during health
        check.
      - Also at startup to fetch `node_info.network`, i.e., the network
        identifier, during health check.
      - Also at startup to assert that `sync_info.catching_up` is false, during
        health check.
  2. To fetch the rollup latest time and height used in many methods, often
    alongside `node_info.network`, for example:
      - As a dependency, because the latest height is necessary in calling the
        `/rollup_params` RPC, during health check.
      - Needed in channel handshake (open try, ack, confirm; close confirm).
      - In connection handshake (open try, ack, confirm), for both the source
        chain (to update client), and destination chain (to construct proofs).
      - Note: It seems like we bombard the node with `/status` queries, but most
        of the queries hit the Hermes in-process cache.
      - For updating clients during everyday IBC relaying. In case there is
        non-zero connection delay, we again bombard the node with `/status`
        queries.

- Status: Nothing available yet, but part of the needed data can be gathered
  from different endpoints like `ledger_getHead` or
  `prover_AggregatedProofData`.

### `/rollup_health`

- Needed for basic check to assess the health of the rollup nodes.
- Only used once, at relayer startup during health check.
- Not needed for IBC relaying strictly speaking.
- In case of Cosmos chains, it returns empty result (200 OK) on success.

- Status: Available as a `/health` method on RPC client

## IBC modules RPC

- Queries client, connection and channel-associated data and states.
- Used to retrieve commitment proofs for every IBC message relayed
- To obtain the client upgraded state, while the relayer is handling chain
  upgrades.

- Status: RPC methods Implemented expect for the upgraded client state query.

### Channel endpoints

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

### Client endpoints

- `ibc_clientStates`: Requests all client states associated with the chain.
- `ibc_ConsensusStates`: Requests all the consensus states associated with a
  specified client.
- `ibc_consensusStateHeights`: Requests all the consensus state heights
  associated with a specified client.
- `ibc_upgradedClientState`: Requests the upgraded client state associated with a
  specified client.

### Connection endpoints

- `ibc_clientConnections`: Requests all connections associated with a specified client.
- ‍`ibc_connections`: Requests all connections associated with the chain.

## Rollup WebSocket

### `/subscribe_aggregated_proofs`

- Subscribe to the rollup node's websocket and listen to aggregated proofs every
  time a proof is written for the rollup.

- Status: Nothing available yet.

### `/subscribe_events`

- (Nice to have) Connect to the rollup node's websocket and subscribes to the
  ibc events.

- Status: The `ledger_subscribeSlots` endpoint provides a stream of
  [`SlotResponse`](https://github.com/Sovereign-Labs/sovereign-sdk/blob/bd469c70fc1227a7785fb177a34de21bb6d5eb08/rollup-interface/src/node/rpc/mod.rs#L136-L149)
  type, encompassing the processed batches. Within this, we gain access to a
  list of transactions and so on related events. However, for obtaining IBC
  events, it is necessary to implement a filter on the SlotResponse type to
  exclude non-IBC batches, txs and events.
