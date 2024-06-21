<div align="center">
    <h1>Sovereign IBC Modules</h1>
</div>

## Overview

The Sovereign IBC modules form a comprehensive suite that integrates `ibc-rs`
with the Sovereign SDK rollups. These modules are purpose-built to empower
Sovereign SDK rollups with a range of essential IBC components and
functionalities, including:

- `sov-ibc`: Serving as the central entrypoint and hub, this module orchestrates
  the integration of IBC core layers such as client, connection, and channel,
  while also managing integrated light clients and applications.

- `sov-ibc-transfer`: This module is dedicated to integrating ICS-20 application
  and handling the intricate IBC transfer functionalities within Sovereign SDK
  rollups. It works hand in hand with the `sov-bank` module for executing ICS-20
  packets.

- `sov-consensus-state-tracker`: Serving as a custom "kernel" module, focuses on
  tracking the consensus state of the Data Availability (DA) layer. This module
  is not an IBC module per se, but it is essential for consistently retrieving
  the DA's consensus state, along with the rollup's latest height and timestamp
  during each slot execution. This ensures that the `sov-ibc` module remains
  synchronized with the latest states, which is vital for `ibc-rs` handlers to
  accurately process incoming packets.
  - **Note**: Currently, this module supports `mock-da` and `celestia-da`.
    Depending on which DA layer is used, the corresponding DA feature flag
    should be enabled.

## Available RPC Methods

The RPC method implementations for the Sovereign IBC modules can be found in
their respective `rpc.rs` file within each module. Additionally, each "normal"
module exposes a set of methods for fetching their Id and status, as follows:

- `ibc_moduleId`
- `ibc_health`
- `transfer_moduleId`
- `transfer_health`

Here is an overview of the RPC methods available for each module:

### `sov-ibc` RPC Methods

#### Client

- `ibc_clientState`
- `ibc_clientStates`
- `ibc_consensusState`
- `ibc_consensusStates`
- `ibc_consensusStateHeights`
- `ibc_clientStatus`
- `ibc_upgradedClientState`
- `ibc_upgradedConsensusState`

#### Connection

- `ibc_connection`
- `ibc_connections`
- `ibc_clientConnections`
- `ibc_connectionClientState`
- `ibc_connectionConsensusState`
- `ibc_connectionParams`

#### Channel

- `ibc_channel`
- `ibc_channels`
- `ibc_connectionChannels`
- `ibc_channelClientState`
- `ibc_channelConsensusState`
- `ibc_packetCommitment`
- `ibc_packetCommitments`
- `ibc_packetReceipt`
- `ibc_packetAcknowledgement`
- `ibc_packetAcknowledgements`
- `ibc_unreceivedPackets`
- `ibc_unreceivedAcks`
- `ibc_nextSequenceReceive`

#### Example

```bash
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"ibc_clientState","params":{"request":{"client_id": "100-sov-celestia-0"}},"id":1}' http://127.0.0.1:12345
```

### `sov-ibc-transfer` RPC Methods

- `transfer_mintedTokenName`: Queries the minted tokens by provided token ID
  and returns the corresponding token name.
- `transfer_mintedTokenId`: Queries the minted tokens by provided token name and
  returns the corresponding token ID.
