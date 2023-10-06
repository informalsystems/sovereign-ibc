use std::cell::RefCell;
use std::rc::Rc;

use anyhow::Result;
use ibc::applications::transfer::context::TokenTransferExecutionContext;
use ibc::applications::transfer::msgs::transfer::MsgTransfer;
use ibc::applications::transfer::packet::PacketData;
use ibc::applications::transfer::{send_transfer, Memo, PrefixedCoin};
use ibc::core::ics04_channel::timeout::TimeoutHeight;
use ibc::core::ics24_host::identifier::{ChannelId, PortId};
use ibc::core::timestamp::Timestamp;
use ibc::core::ExecutionContext;
use ibc::Signer;
use sov_modules_api::{Context, WorkingSet};

use super::context::EscrowExtraData;
use super::Transfer;

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    schemars(bound = "C::Address: ::schemars::JsonSchema", rename = "CallMessage")
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq)]
pub struct SDKTokenTransfer<C: Context> {
    /// the port on which the packet will be sent
    pub port_id_on_a: PortId,
    /// the channel by which the packet will be sent
    pub chan_id_on_a: ChannelId,
    /// Timeout height relative to the current block height.
    /// The timeout is disabled when set to None.
    pub timeout_height_on_b: TimeoutHeight,
    /// Timeout timestamp relative to the current block timestamp.
    /// The timeout is disabled when set to 0.
    pub timeout_timestamp_on_b: Timestamp,
    /// The address of the token to be sent
    pub token_address: C::Address,
    /// The amount of tokens sent
    pub amount: sov_bank::Amount,
    /// The address of the token sender
    pub sender: Signer,
    /// The address of the token receiver on the counterparty chain
    pub receiver: Signer,
    /// Additional note associated with the message
    pub memo: Memo,
}

impl<C: Context> Transfer<C> {
    pub fn transfer(
        &self,
        sdk_token_transfer: SDKTokenTransfer<C>,
        execution_context: &mut impl ExecutionContext,
        token_ctx: &mut impl TokenTransferExecutionContext<EscrowExtraData<C>>,
        working_set: Rc<RefCell<&mut WorkingSet<C>>>,
    ) -> Result<sov_modules_api::CallResponse> {
        let msg_transfer: MsgTransfer = {
            let denom = {
                let token_name = self
                    .bank
                    .get_token_name(
                        &sdk_token_transfer.token_address,
                        &mut working_set.borrow_mut(),
                    )
                    .ok_or(anyhow::anyhow!(
                        "Token with address {} doesn't exist",
                        sdk_token_transfer.token_address
                    ))?;

                if self.token_was_created_by_ibc(
                    &token_name,
                    &sdk_token_transfer.token_address,
                    &mut working_set.borrow_mut(),
                ) {
                    // The token was created by the IBC module, and the ICS-20
                    // denom was stored in the token name. Hence, we need to use
                    // the token name as denom.
                    token_name
                } else {
                    // This applies to all other tokens created on this
                    // sovereign SDK chain. The token name is not guaranteed to
                    // be unique, and hence we must use the token address (which
                    // is guaranteed to be unique) as the ICS-20 denom to ensure
                    // uniqueness.
                    sdk_token_transfer.token_address.to_string()
                }
            };

            MsgTransfer {
                port_id_on_a: sdk_token_transfer.port_id_on_a,
                chan_id_on_a: sdk_token_transfer.chan_id_on_a,
                packet_data: PacketData {
                    token: PrefixedCoin {
                        denom: denom
                            .parse()
                            .map_err(|_err| anyhow::anyhow!("Failed to parse denom {denom}"))?,
                        amount: sdk_token_transfer.amount.into(),
                    },
                    sender: sdk_token_transfer.sender,
                    receiver: sdk_token_transfer.receiver,
                    memo: sdk_token_transfer.memo,
                },
                timeout_height_on_b: sdk_token_transfer.timeout_height_on_b,
                timeout_timestamp_on_b: sdk_token_transfer.timeout_timestamp_on_b,
            }
        };

        send_transfer(
            execution_context,
            token_ctx,
            msg_transfer,
            &EscrowExtraData {
                token_address: sdk_token_transfer.token_address,
            },
        )?;

        Ok(sov_modules_api::CallResponse::default())
    }

    /// This function returns true if the token to be sent was created by IBC.
    /// This only occurs for tokens that are native to and received from other
    /// chains; i.e. for tokens for which this chain isn't the source.
    fn token_was_created_by_ibc(
        &self,
        token_name: &String,
        token_address: &C::Address,
        working_set: &mut WorkingSet<C>,
    ) -> bool {
        match self.minted_tokens.get(token_name, working_set) {
            Some(minted_token_address) => minted_token_address == *token_address,
            None => false,
        }
    }
}
