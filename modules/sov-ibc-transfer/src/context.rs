use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

use ibc_app_transfer::context::{TokenTransferExecutionContext, TokenTransferValidationContext};
use ibc_app_transfer::module::{
    on_acknowledgement_packet_validate, on_chan_open_ack_validate, on_chan_open_confirm_validate,
    on_chan_open_init_execute, on_chan_open_init_validate, on_chan_open_try_execute,
    on_chan_open_try_validate, on_recv_packet_execute, on_timeout_packet_execute,
    on_timeout_packet_validate,
};
use ibc_app_transfer::types::error::TokenTransferError;
use ibc_app_transfer::types::{
    Amount, Memo, PrefixedCoin, PrefixedDenom, TracePrefix, PORT_ID_STR, VERSION,
};
use ibc_core::channel::types::acknowledgement::Acknowledgement;
use ibc_core::channel::types::channel::{Counterparty, Order};
use ibc_core::channel::types::error::{ChannelError, PacketError};
use ibc_core::channel::types::packet::Packet;
use ibc_core::channel::types::Version as ChannelVersion;
use ibc_core::host::types::identifiers::{ChannelId, ConnectionId, PortId};
use ibc_core::primitives::Signer;
use ibc_core::router::module::Module;
use ibc_core::router::types::module::ModuleExtras;
use sov_bank::{Coins, IntoPayable, Payable, TokenId};
use sov_modules_api::{Context, Spec, WorkingSet};
use uint::FromDecStrErr;

use super::IbcTransfer;
use crate::utils::compute_escrow_address;

/// Using a different salt will result in a different token address. Since
/// ICS-20 tokens coming from other chains are guaranteed to have unique names,
/// we don't need to use different salt values, thus we just use 0.
const SALT: u64 = 0u64;

/// Maximum memo length allowed for ICS-20 transfers. This bound corresponds to
/// the `MaximumMemoLength` in the `ibc-go`
const MAXIMUM_MEMO_LENGTH: usize = 32768; // 1 << 15

/// We need to create a wrapper around the `Transfer` module and `WorkingSet`,
/// because we only get the `WorkingSet` at call-time from the Sovereign SDK,
/// which must be passed to `TokenTransferValidationContext` methods through
/// the `self` argument.
pub struct IbcTransferContext<'ws, S: Spec> {
    pub ibc_transfer: IbcTransfer<S>,
    pub sdk_context: Context<S>,
    pub working_set: Rc<RefCell<&'ws mut WorkingSet<S>>>,
}

impl<'ws, S: Spec> IbcTransferContext<'ws, S> {
    pub fn new(
        ibc_transfer: IbcTransfer<S>,
        sdk_context: Context<S>,
        working_set: Rc<RefCell<&'ws mut WorkingSet<S>>>,
    ) -> Self {
        Self {
            ibc_transfer,
            sdk_context,
            working_set,
        }
    }

    /// Stores mapping from "denom to token ID" and vice versa for an
    /// IBC-created token.
    fn record_minted_token(&self, token_id: TokenId, token_name: String) {
        self.ibc_transfer.minted_token_id_to_name.set(
            &token_id,
            &token_name,
            *self.working_set.borrow_mut(),
        );

        self.ibc_transfer.minted_token_name_to_id.set(
            &token_name,
            &token_id,
            *self.working_set.borrow_mut(),
        );
    }

    /// Validate that the token is native and **not** an IBC-created token by
    /// cross-referencing with the `minted_token_id_to_name` state. If a token
    /// found and the token name starts with the trace path
    /// `<given_port_id>/<given_channel_id>/`, returns an error, otherwise
    /// returns the token ID.
    ///
    /// NOTE: In un-escrowing scenarios there is no way to make this fail from
    /// an honest counterparty chain, as this method only fails when the
    /// counterparty chain produces a malicious IBC transfer `send_packet()`.
    fn get_native_token_id(
        &self,
        coin: &PrefixedCoin,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<TokenId, TokenTransferError> {
        let token_id = TokenId::from_str(&coin.denom.to_string()).map_err(|_| {
            TokenTransferError::InvalidCoin {
                coin: coin.to_string(),
            }
        })?;

        if let Some(token_name) = self
            .ibc_transfer
            .minted_token_id_to_name
            .get(&token_id, *self.working_set.borrow_mut())
        {
            let prefixed_denom = PrefixedDenom::from_str(&token_name).map_err(|e| {
                TokenTransferError::Other(format!(
                    "Failed to parse token name: {token_name} with error: {e}"
                ))
            })?;

            let trace_prefix = TracePrefix::new(port_id.clone(), channel_id.clone());

            if prefixed_denom.trace_path.starts_with(&trace_prefix) {
                return Err(TokenTransferError::Other(format!(
                    "Token with ID '{token_id}' is an IBC-created token and cannot be transferred.\
                    Use '{token_name}' as denom for transferring an IBC token to the source chain"
                )));
            }
        }

        Ok(token_id)
    }

    /// Validate that the token is an IBC-created token by cross-referencing
    /// with the `minted_token_name_to_id` state. If a token is found, returns
    /// the token ID, otherwise returns an error.
    fn get_ibc_token_id(&self, coin: &PrefixedCoin) -> Result<TokenId, TokenTransferError> {
        self.ibc_transfer
            .minted_token_name_to_id
            .get(&coin.denom.to_string(), *self.working_set.borrow_mut())
            .ok_or(TokenTransferError::InvalidCoin {
                coin: coin.to_string(),
            })
    }

    /// Obtains the escrow address for a given port and channel pair by looking
    /// up the cache. If the cache does not contain the address, it is computed
    /// and stored in the cache.
    fn obtain_escrow_address(&self, port_id: &PortId, channel_id: &ChannelId) -> S::Address {
        let mut working_set = self.working_set.borrow_mut();

        self.ibc_transfer
            .escrow_address_cache
            .get(&(port_id.clone(), channel_id.clone()), *working_set)
            .unwrap_or_else(|| {
                let escrow_account = compute_escrow_address::<S>(port_id, channel_id);
                self.ibc_transfer.escrow_address_cache.set(
                    &(port_id.clone(), channel_id.clone()),
                    &escrow_account,
                    *working_set,
                );
                escrow_account
            })
    }

    /// Validates that the sender has sufficient balance to perform the
    /// transfer. If the user has sufficient balance, returns the balance,
    /// otherwise returns an error.
    fn validate_balance(
        &self,
        token_id: TokenId,
        address: &S::Address,
        amount: Amount,
    ) -> Result<Amount, TokenTransferError> {
        let sender_balance: u64 = self
            .ibc_transfer
            .bank
            .get_balance_of(
                address.as_token_holder(),
                token_id,
                *self.working_set.borrow_mut(),
            )
            .ok_or(TokenTransferError::Other(format!(
                "No balance for token with ID: '{token_id}'"
            )))?;

        let sender_balance = sender_balance.into();

        if amount > sender_balance {
            return Err(TokenTransferError::InsufficientFunds {
                send_attempt: sender_balance.to_string(),
                available_funds: amount.to_string(),
            });
        }

        Ok(sender_balance)
    }

    /// Creates a new token with the specified `token_name` and mints an initial
    /// balance to the `minter_address`.
    ///
    /// Note: The mint authority must be held by the `IbcTransfer` module, so
    /// the `authorized_minters` is set to the `IbcTransfer` address. Also,
    /// remember that the `token_name` is a denom prefixed with IBC and
    /// originates from the counterparty chain.
    fn create_token(
        &self,
        token_name: String,
        minter_address: &S::Address,
    ) -> Result<TokenId, TokenTransferError> {
        // Make sure to use `ibc_transfer` address as the sender
        let context = Context::new(
            // TODO(rano): This should be the `ibc_transfer` address
            minter_address.clone(),
            self.sdk_context.sequencer().clone(),
            self.sdk_context.visible_slot_number(),
        );

        let new_token_id = self
            .ibc_transfer
            .bank
            .create_token(
                token_name.clone(),
                SALT,
                0,
                minter_address.as_token_holder(),
                vec![self.ibc_transfer.id.to_payable()],
                &context,
                &mut self.working_set.borrow_mut(),
            )
            .map_err(|err| TokenTransferError::Other(err.to_string()))?;

        self.record_minted_token(new_token_id, token_name);

        Ok(new_token_id)
    }

    /// Transfers `amount` tokens from `from_account` to `to_account`
    fn transfer(
        &self,
        token_id: TokenId,
        from_account: &S::Address,
        to_account: &S::Address,
        amount: &Amount,
    ) -> Result<(), TokenTransferError> {
        let amount: sov_bank::Amount = (*amount.as_ref())
            .try_into()
            .map_err(|_| TokenTransferError::InvalidAmount(FromDecStrErr::InvalidLength))?;
        let coin = Coins { amount, token_id };

        self.ibc_transfer
            .bank
            .transfer_from(
                from_account,
                to_account,
                coin,
                *self.working_set.borrow_mut(),
            )
            .map_err(|err| TokenTransferError::Other(err.to_string()))?;

        Ok(())
    }
}

impl<'ws, S> core::fmt::Debug for IbcTransferContext<'ws, S>
where
    S: Spec,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransferContext")
            .field("transfer_mod", &self.ibc_transfer)
            .finish()
    }
}

impl<'ws, S> TokenTransferValidationContext for IbcTransferContext<'ws, S>
where
    S: Spec,
{
    type AccountId = Address<S>;

    fn get_port(&self) -> Result<PortId, TokenTransferError> {
        PortId::new(PORT_ID_STR.to_string()).map_err(TokenTransferError::InvalidIdentifier)
    }

    fn can_send_coins(&self) -> Result<(), TokenTransferError> {
        Ok(())
    }

    fn can_receive_coins(&self) -> Result<(), TokenTransferError> {
        Ok(())
    }

    fn mint_coins_validate(
        &self,
        _account: &Self::AccountId,
        _coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError> {
        // We can always mint
        Ok(())
    }

    /// Any token that is to be burned will have been previously minted, so we
    /// can expect to find the token ID in our `minted_token_name_to_id` map.
    ///
    /// This is called in a `send_transfer()` in the case where we are NOT the
    /// token source
    fn burn_coins_validate(
        &self,
        account: &Self::AccountId,
        coin: &PrefixedCoin,
        memo: &Memo,
    ) -> Result<(), TokenTransferError> {
        // Disallowing large memos to prevent potential overloads on the system.
        if memo.as_ref().len() > MAXIMUM_MEMO_LENGTH {
            return Err(TokenTransferError::Other(format!(
                "Memo size exceeds maximum allowed length: {MAXIMUM_MEMO_LENGTH}"
            )));
        }

        let minted_token_id = self.get_ibc_token_id(coin)?;

        self.validate_balance(minted_token_id, &account.address, coin.amount)?;

        Ok(())
    }

    /// This is called in a `send_transfer()` in the case where we are the token source
    fn escrow_coins_validate(
        &self,
        from_account: &Self::AccountId,
        port_id: &PortId,
        channel_id: &ChannelId,
        coin: &PrefixedCoin,
        memo: &Memo,
    ) -> Result<(), TokenTransferError> {
        // Disallowing large memos to prevent potential overloads on the system.
        if memo.as_ref().len() > MAXIMUM_MEMO_LENGTH {
            return Err(TokenTransferError::Other(format!(
                "Memo size exceeds maximum allowed length: {MAXIMUM_MEMO_LENGTH}"
            )));
        }

        let token_id = self.get_native_token_id(coin, port_id, channel_id)?;

        self.validate_balance(token_id, &from_account.address, coin.amount)?;

        Ok(())
    }

    /// This is called in a `recv_packet()` in the case where we are the token
    /// source.
    ///
    /// Note: ibc-rs strips the first prefix upon receival. That is, if token
    /// with denom `my_token` was previously sent on channel `channel-1` and
    /// port `transfer` (on the counterparty), it will be received in
    /// `recv_packet` as `transfer/channel-1/my_token`. However, ibc-rs strips
    /// `transfer/channel-1/` off the denom before passing it here, such that
    /// `coin.denom` would be `my_token`.
    ///
    /// This is especially important for us, as we use the denom to lookup the
    /// token ID. Hence, we need to be careful not to use `my_token` in some
    /// instances and `transfer/channel-1/my_token` in others. Fortunately,
    /// ibc-rs solves that problem for us.
    fn unescrow_coins_validate(
        &self,
        _to_account: &Self::AccountId,
        port_id: &PortId,
        channel_id: &ChannelId,
        coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError> {
        let token_id = self.get_native_token_id(coin, port_id, channel_id)?;

        let escrow_address = self.obtain_escrow_address(port_id, channel_id);

        self.validate_balance(token_id, &escrow_address, coin.amount)?;

        Ok(())
    }
}

impl<'ws, S: Spec> TokenTransferExecutionContext for IbcTransferContext<'ws, S> {
    /// This is called in a `recv_packet()` in the case where we are NOT the
    /// token source.
    fn mint_coins_execute(
        &mut self,
        account: &Self::AccountId,
        coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError> {
        // 1. if token ID doesn't exist in `minted_token_name_to_id`, then
        //    create a new token and store in the maps
        let token_id = match self.get_ibc_token_id(coin) {
            Ok(token_id) => token_id,
            Err(_) => self.create_token(coin.denom.to_string(), &account.address)?,
        };

        // 2. mint tokens
        let amount: sov_bank::Amount = (*coin.amount.as_ref())
            .try_into()
            .map_err(|_| TokenTransferError::InvalidAmount(FromDecStrErr::InvalidLength))?;
        let sdk_coins = Coins { amount, token_id };

        self.ibc_transfer
            .bank
            .mint(
                &sdk_coins,
                &account.address,
                self.ibc_transfer.id.to_payable(),
                &mut self.working_set.borrow_mut(),
            )
            .map_err(|err| TokenTransferError::Other(err.to_string()))?;

        Ok(())
    }

    /// This is called in a `send_transfer()` in the case where we are NOT the
    /// token source
    fn burn_coins_execute(
        &mut self,
        account: &Self::AccountId,
        coin: &PrefixedCoin,
        _memo: &Memo,
    ) -> Result<(), TokenTransferError> {
        // The token was created by the IBC module, and the ICS-20 denom was
        // stored in the token name. Hence, we need to use the token name as
        // denom.
        let token_id = self.get_ibc_token_id(coin)?;

        let amount: sov_bank::Amount = (*coin.amount.as_ref())
            .try_into()
            .map_err(|_| TokenTransferError::InvalidAmount(FromDecStrErr::InvalidLength))?;
        let sdk_coins = Coins { amount, token_id };

        self.ibc_transfer
            .bank
            .burn(
                sdk_coins,
                &account.address,
                &mut self.working_set.borrow_mut(),
            )
            .map_err(|err| TokenTransferError::Other(err.to_string()))?;

        Ok(())
    }

    /// This is called in a `send_transfer()` in the case where we are the token source
    fn escrow_coins_execute(
        &mut self,
        from_account: &Self::AccountId,
        port_id: &PortId,
        channel_id: &ChannelId,
        coin: &PrefixedCoin,
        _memo: &Memo,
    ) -> Result<(), TokenTransferError> {
        // The token name on the Sovereign SDK chains is not guaranteed to be
        // unique, and hence we must use the token ID (which is guaranteed to be
        // unique) as the ICS-20 denom to ensure uniqueness.
        let token_id = TokenId::from_str(&coin.denom.to_string()).map_err(|_| {
            TokenTransferError::InvalidCoin {
                coin: coin.to_string(),
            }
        })?;

        let escrow_account = self.obtain_escrow_address(port_id, channel_id);

        // transfer coins to escrow account
        self.transfer(
            token_id,
            &from_account.address,
            &escrow_account,
            &coin.amount,
        )?;

        Ok(())
    }

    /// This is called in a `recv_packet()` in the case where we are the token source.
    ///
    /// For more details, see note in `unescrow_coins_validate()`.
    fn unescrow_coins_execute(
        &mut self,
        to_account: &Self::AccountId,
        port_id: &PortId,
        channel_id: &ChannelId,
        coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError> {
        let token_id = TokenId::from_str(&coin.denom.to_string()).map_err(|_| {
            TokenTransferError::InvalidCoin {
                coin: coin.to_string(),
            }
        })?;

        let escrow_account = self.obtain_escrow_address(port_id, channel_id);

        // transfer coins out of escrow account to `to_account`
        self.transfer(token_id, &escrow_account, &to_account.address, &coin.amount)?;

        Ok(())
    }
}

/// Address type, which wraps C::Address. This is needed to implement
/// `TryFrom<Signer>` (circumventing the orphan rule).
pub struct Address<S: Spec> {
    pub address: S::Address,
}

impl<S: Spec> TryFrom<Signer> for Address<S> {
    type Error = anyhow::Error;

    fn try_from(signer: Signer) -> Result<Self, Self::Error> {
        Ok(Address {
            address: signer.as_ref().parse().map_err(|_| {
                anyhow::anyhow!("Failed to parse signer address: {}", signer.as_ref())
            })?,
        })
    }
}

impl<'ws, S: Spec> Module for IbcTransferContext<'ws, S> {
    fn on_chan_open_init_validate(
        &self,
        order: Order,
        connection_hops: &[ConnectionId],
        port_id: &PortId,
        channel_id: &ChannelId,
        counterparty: &Counterparty,
        version: &ChannelVersion,
    ) -> Result<ChannelVersion, ChannelError> {
        on_chan_open_init_validate(
            self,
            order,
            connection_hops,
            port_id,
            channel_id,
            counterparty,
            version,
        )
        .map_err(|e: TokenTransferError| ChannelError::AppModule {
            description: e.to_string(),
        })?;

        Ok(ChannelVersion::new(VERSION.to_string()))
    }

    fn on_chan_open_init_execute(
        &mut self,
        order: Order,
        connection_hops: &[ConnectionId],
        port_id: &PortId,
        channel_id: &ChannelId,
        counterparty: &Counterparty,
        version: &ChannelVersion,
    ) -> Result<(ModuleExtras, ChannelVersion), ChannelError> {
        on_chan_open_init_execute(
            self,
            order,
            connection_hops,
            port_id,
            channel_id,
            counterparty,
            version,
        )
        .map_err(|e: TokenTransferError| ChannelError::AppModule {
            description: e.to_string(),
        })
    }

    fn on_chan_open_try_validate(
        &self,
        order: Order,
        connection_hops: &[ConnectionId],
        port_id: &PortId,
        channel_id: &ChannelId,
        counterparty: &Counterparty,
        counterparty_version: &ChannelVersion,
    ) -> Result<ChannelVersion, ChannelError> {
        on_chan_open_try_validate(
            self,
            order,
            connection_hops,
            port_id,
            channel_id,
            counterparty,
            counterparty_version,
        )
        .map_err(|e: TokenTransferError| ChannelError::AppModule {
            description: e.to_string(),
        })?;
        Ok(ChannelVersion::new(VERSION.to_string()))
    }

    fn on_chan_open_try_execute(
        &mut self,
        order: Order,
        connection_hops: &[ConnectionId],
        port_id: &PortId,
        channel_id: &ChannelId,
        counterparty: &Counterparty,
        counterparty_version: &ChannelVersion,
    ) -> Result<(ModuleExtras, ChannelVersion), ChannelError> {
        on_chan_open_try_execute(
            self,
            order,
            connection_hops,
            port_id,
            channel_id,
            counterparty,
            counterparty_version,
        )
        .map_err(|e: TokenTransferError| ChannelError::AppModule {
            description: e.to_string(),
        })
    }

    fn on_chan_open_ack_validate(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        counterparty_version: &ChannelVersion,
    ) -> Result<(), ChannelError> {
        on_chan_open_ack_validate(self, port_id, channel_id, counterparty_version).map_err(
            |e: TokenTransferError| ChannelError::AppModule {
                description: e.to_string(),
            },
        )
    }

    fn on_chan_open_ack_execute(
        &mut self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _counterparty_version: &ChannelVersion,
    ) -> Result<ModuleExtras, ChannelError> {
        Ok(ModuleExtras::empty())
    }

    fn on_chan_open_confirm_validate(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<(), ChannelError> {
        on_chan_open_confirm_validate(self, port_id, channel_id).map_err(|e: TokenTransferError| {
            ChannelError::AppModule {
                description: e.to_string(),
            }
        })
    }

    fn on_chan_open_confirm_execute(
        &mut self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
    ) -> Result<ModuleExtras, ChannelError> {
        Ok(ModuleExtras::empty())
    }

    fn on_chan_close_init_validate(
        &self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
    ) -> Result<(), ChannelError> {
        Ok(())
    }

    fn on_chan_close_init_execute(
        &mut self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
    ) -> Result<ModuleExtras, ChannelError> {
        Ok(ModuleExtras::empty())
    }

    fn on_chan_close_confirm_validate(
        &self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
    ) -> Result<(), ChannelError> {
        Ok(())
    }

    fn on_chan_close_confirm_execute(
        &mut self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
    ) -> Result<ModuleExtras, ChannelError> {
        Ok(ModuleExtras::empty())
    }

    fn on_recv_packet_execute(
        &mut self,
        packet: &Packet,
        _relayer: &Signer,
    ) -> (ModuleExtras, Acknowledgement) {
        on_recv_packet_execute(self, packet)
    }

    fn on_acknowledgement_packet_validate(
        &self,
        packet: &Packet,
        acknowledgement: &Acknowledgement,
        relayer: &Signer,
    ) -> Result<(), PacketError> {
        on_acknowledgement_packet_validate(self, packet, acknowledgement, relayer).map_err(
            |e: TokenTransferError| PacketError::AppModule {
                description: e.to_string(),
            },
        )
    }

    fn on_acknowledgement_packet_execute(
        &mut self,
        _packet: &Packet,
        _acknowledgement: &Acknowledgement,
        _relayer: &Signer,
    ) -> (ModuleExtras, Result<(), PacketError>) {
        (ModuleExtras::empty(), Ok(()))
    }

    /// Note: `MsgTimeout` and `MsgTimeoutOnClose` use the same callback
    fn on_timeout_packet_validate(
        &self,
        packet: &Packet,
        relayer: &Signer,
    ) -> Result<(), PacketError> {
        on_timeout_packet_validate(self, packet, relayer).map_err(|e: TokenTransferError| {
            PacketError::AppModule {
                description: e.to_string(),
            }
        })
    }

    /// Note: `MsgTimeout` and `MsgTimeoutOnClose` use the same callback
    fn on_timeout_packet_execute(
        &mut self,
        packet: &Packet,
        relayer: &Signer,
    ) -> (ModuleExtras, Result<(), PacketError>) {
        let res = on_timeout_packet_execute(self, packet, relayer);
        (
            res.0,
            res.1
                .map_err(|e: TokenTransferError| PacketError::AppModule {
                    description: e.to_string(),
                }),
        )
    }
}
