# ADR 001 - ICS-20 Transfer Module Implementation

## Changelog

- 2024-04-10: ADR Drafted

## Status

Implemented

## Context

This ADR outlines the implementation of the ICS-20 transfer module within the
Sovereign SDK system using the `ibc-rs` library.

we established a module struct to integrate the ICS-20 implementation from
`ibc-rs` into the Sovereign SDK, particularly to implement the consumer traits
of `ibc-rs`. This struct's primary role is to provide keys for accessing storage
to get/set IBC relevant states. We named this struct `IbcTransfer`, residing
within its own standalone crate called `sov-ibc-transfer`. It is annotated with
the `ModuleInfo` derive, which registers `IbcTransfer` as a module within the
Sovereign SDK system.

It's important to note that the `IbcTransfer` module was developed independently
from `Ibc`. In the context of Sovereign SDK, each struct implementing
`ModuleInfo` represents a distinct module within the system. This modular
approach allows for more efficient integration, initialization, and operation of
modules. Initially, we considered combining these modules under a single entity,
but that approach would complicate dependency and feature management. (See PR#14
for more details)

## Entry Point

Despite comprising two distinct modules, the entry point for both is `Ibc`
module. This module plays a pivotal role in handling incoming IBC messages. This
necessity stems from the operational structure of `ibc-rs` handlers, where the
processing of ICS-20 packets relies on core handlers accessible exclusively
through `Ibc`.

Upon receiving a message, `Ibc` proceeds to identify its type, such as ICS-20
packets, and directs it to the corresponding module. Specifically, in our setup,
this entails invoking the
[`transfer`](https://github.com/informalsystems/sovereign-ibc/blob/0c3b99f44613ff9a8668ade798b39507a17a7321/modules/sov-ibc/src/call.rs#L66)
method to route the ICS-20 messages to the `IbcTransfer` for subsequent handling
and processing.

## Module Structure

In addition to its core functionalities, `IbcTransfer` maintains a record of
tokens minted on a rollup created by the transfer module. You can see details of
this structure
[here](https://github.com/informalsystems/sovereign-ibc/blob/4e37dc4bb88624765384d1662549c00e991acc4a/modules/sov-ibc-transfer/src/lib.rs#L20-L50)
in the `sov-ibc-transfer` crate.

Specifically, `IbcTransfer` manages two essential maps:

- `minted_token_name_to_id`: This map links the token name to its corresponding
  token ID for tokens created by IBC. It is used during the minting and burning
  processes to check if the token exists and to obtain the necessary ID for
  these operations.

- `minted_token_id_to_name`: This map connects the token ID to its corresponding
  token name for tokens created by IBC. It is utilized during escrow and
  un-escrow processes to confirm that the `TokenId` obtained from the `denom` is
  **not** an IBC-created token, indicating it is a native token for escrow and
  un-escrow operations.

If the purpose of these maps is not entirely clear, reviewing how each transfer
scenario is handled in the following section will provide clarity.

## Transfer Scenarios

Given the context provided, the `IbcTransfer` module handles four scenarios,
each comprising validation _(x_validate method)_ and execution _(x_execute
method)_ stages.

Before diving into the specifics of each scenario, it is essential to understand
that the token name on the Sovereign SDK rollups is not guaranteed to be unique,
and hence when transferring **native tokens** we must use the token ID (which is
guaranteed to be unique) as the ICS-20 denom to ensure uniqueness.

### Escrowing Tokens - Sender on Rollup with Rollup as Source

1. Verify that the `memo` field does not exceed the maximum allowed length to
   prevent large memos from overwhelming the system. We set the maximum memo
   length to 32768 bytes like the `ibc-go`.
2. Identify the token ID by parsing the `denom` field of the receiving
   `MsgTransfer`.
3. Validate that the token is native and **not** an IBC-created token by
   cross-referencing with the `minted_token_id_to_name` state.
   - If the token ID is found in the `minted_token_id_to_name` state, check if
     the corresponding token name begins with the trace path
     `<given_port_id>/<given_channel_id>/`. If it does, reject the transfer.
4. Confirm the sender has a sufficient balance.
5. Retrieve the escrow address for the specified port and channel pair from the
   cache. If absent, compute and cache the address. Utilize caching to avoid
   repeated computations.
6. Execute the transfer function of the `bank` module to escrow tokens into the
   designated escrow account.

### Unescrowing Tokens - Receiver on Rollup with Rollup as Source

1. Obtain the base denom by removing the prefix. In the receiving process,
   `ibc-rs` automatically removes the first prefix. For instance, if a token
   with the denom `my_token` was previously sent on channel `channel-0` and port
   `transfer` (on the counterparty), it will be received in `recv_packet` as
   `transfer/channel-0/my_token`. `ibc-rs` strips `transfer/channel-0/` from the
   denom, so `coin.denom` would be `my_token` for unescrowing.
2. Validate that the token is native and **not** an IBC-created token by
   referencing the `minted_token_id_to_name` state.
   - If the token ID is found in the `minted_token_id_to_name` state, check if
     the corresponding token name begins with the trace path
     `<given_port_id>/<given_channel_id>/`. If it does, reject the transfer.
     This step only fails when the counterparty chain produces a malicious IBC
     transfer `send_packet()`.
3. Obtain the escrow address for a specified port and channel pair, similar to
   the escrowing step.
4. Verify that the escrow account has a sufficient balance.
5. Unescrow the token from the escrow account to the receiver's address.

### Minting Tokens - Receiver on Rollup with Sender as Source

1. Obtain the full denom by prefixing it (e.g., `transfer/channel-0/uatom`).
   `ibc-rs` handles prefixing a base denom with the specified port and channel
   IDs.
2. Retrieve the token ID by checking if a token for the given denom has been
   previously created by the IBC module using the `minted_token_name_to_id` map.
   - If yes, use that `TokenId`.
   - If no,
     [create a new token](https://github.com/informalsystems/sovereign-ibc/blob/4e37dc4bb88624765384d1662549c00e991acc4a/modules/sov-ibc-transfer/src/context.rs#L105)
     with the name set to the `denom`, obtain the token ID, and store the pair
     of _token name_, _token ID_, and its flip in the `minted_token_id_to_name`
     and `minted_token_name_to_id` state maps.
   - NOTE: When IBC initiates the creation of a new token, the `IbcTransfer`
     address is designated as the authorized minter.
   - NOTE: In these steps, we ensure that the `context` object needed for token
     creation uses the `ibc_transfer` address as the `sender` by constructing a
     new context object.
3. Mint tokens to the receiver's address with the specified amount in the
   `MsgRecvPacket` message.

### Burning Tokens - Sender on Rollup with Receiver as Source

1. Verify that the `memo` field does not exceed the maximum allowed length to
   prevent large memos from overwhelming the system. We set the maximum memo
   length to 32768 bytes like the `ibc-go`.
2. Obtain the `TokenId` using the denom from the `minted_token_name_to_id` map.
   - If the token ID is not found, the transfer is rejected.
3. Confirm that the sender has a sufficient balance.
4. Burn tokens from the sender's address by calling `burn` on the `bank` module.

Therefore, As a primary rule, when transferring a native token, the token ID is
used as the denom. For IBC-created tokens, the regular prefixed denomination is
utilized as the denom. This may pose a challenge for front-ends to correctly
identify the token type when crafting the related appropriate transfer message.
Here existing RPC methods can assist.

## Concrete Scenario among Three Sovereign Rollups

Let's assume that the ICS20 application is deployed on three Sovereign rollups
`sovA`, `sovB`, and `sovC`; and they are interconnected as shown below:

```
          chAB┌────┐chAC
      ┌───────┤sovA├───────┐
      │       └────┘       │
      │                    │
chBA┌─┴──┐chBC      chCB┌──┴─┐chCA
    │sovB├──────────────┤sovC│
    └────┘              └────┘
```

We will consider the scenarios when sending `tokA` (a token native to `sovA`) in
this route:

`sovA` -> `sovB` -> `sovC` -> `sovA` -> `sovC` -> `sovB` -> `sovA`

That is, we do a round trip of `tokA` starting and ending at `sovA` via `sovB`
and `sovC`; and then we unwind the round trip.

The following table shows the mappings between Sovereign native tokens and IBC
denom traces for each scenario:

| source rollup | source channel | denom in `MsgTransfer` and denom in ICS20 packet | is target source? | native token on target |  ibc denom trace on target   |
| :-----------: | :------------: | :----------------------------------------------: | :---------------: | :--------------------: | :--------------------------: |
|    `sovA`     |     `chAB`     |                      `tokA`                      |        no         |       `tokA_onB`       |     `transfer/chBA/tokA`     |
|    `sovB`     |     `chBC`     |                    `tokA_onB`                    |        no         |     `tokA_onB_onC`     |   `transfer/chCB/tokA_onB`   |
|    `sovC`     |     `chCA`     |                  `tokA_onB_onC`                  |        no         |   `tokA_onB_onC_onA`   | `transfer/chAC/tokA_onB_onC` |
|    `sovA`     |     `chAC`     |           `transfer/chAC/tokA_onB_onC`           |        yes        |     `tokA_onB_onC`     |   `transfer/chCB/tokA_onB`   |
|    `sovC`     |     `chCB`     |             `transfer/chCB/tokA_onB`             |        yes        |       `tokA_onB`       |     `transfer/chBA/tokA`     |
|    `sovB`     |     `chBA`     |               `transfer/chBA/tokA`               |        yes        |         `tokA`         |              -               |

Note that, `MsgTransfer` on the Sovereign `IBC` module takes an IBC denom trace
when sending it back via its originating channel, otherwise, it takes a native
token. This means that _mint_ and _burn_ methods take an IBC denom trace, while
_escrow_ and _unescrow_ methods take a native token.

|  method  | denom type |    trigger    |                             condition                             |
| :------: | :--------: | :-----------: | :---------------------------------------------------------------: |
|   mint   |    ibc     | `recv_packet` |                                 -                                 |
|   burn   |    ibc     | `MsgTransfer` |        IBC denom must originate from the current channel*         |
|  escrow  |   native   | `MsgTransfer` | corresponding IBC denom can't originate from the current channel* |
| unescrow |   native   | `recv_packet` |                                 -                                 |

(*_the current channel_: the channel where the ICS20 packet will be sent to.)

## Available RPC Methods

To facilitate the interaction with the `IbcTransfer` module, two RPC methods are
available:

- `transfer_mintedTokenName`: Queries the `minted_token_id_to_name` state to
  retrieve the token name for a given token ID.
- `transfer_mintedTokenId`: Queries the `minted_token_name_to_id` state to
  obtain the token ID for a given token name.

Additionally, worth noting there is an RPC method as `transfer_moduleId` that
returns the address of the `IbcTransfer` module.

## References

Here are a list of relevant issues and PRs:

- Review `sov-ibc-transfer` implementation and apply fixes
  [#133](https://github.com/informalsystems/sovereign-ibc/pull/133)
- Token transfer escrow/unescrow + mint/burn tests
  [#47](https://github.com/informalsystems/sovereign-ibc/pull/47)
- Split `sov-ibc` from `sov-ibc-transfer`
  [#14](https://github.com/informalsystems/sovereign-ibc/pull/14)
