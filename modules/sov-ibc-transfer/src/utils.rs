use ibc_app_transfer::types::VERSION;
use ibc_core::host::types::identifiers::{ChannelId, PortId};
use serde::{Deserialize, Serialize};
use sov_bank::TokenId;
use sov_modules_api::digest::Digest;
use sov_modules_api::{CryptoSpec, Spec};

/// The high-level memo type for the Sovereign SDK rollups.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, derive_more::From)]
pub struct SovereignMemo {
    /// the ICS-20 transfer namespace
    pub transfer: TransferMemo,
}

impl SovereignMemo {
    pub fn new(token_id: TokenId) -> Self {
        Self {
            transfer: TransferMemo::new(token_id),
        }
    }

    pub fn token_id(&self) -> TokenId {
        self.transfer.token_id
    }
}

/// The memo type for the ICS-20 transfer module.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, derive_more::From)]
pub struct TransferMemo {
    pub token_id: TokenId,
}

impl TransferMemo {
    pub fn new(token_id: TokenId) -> Self {
        Self { token_id }
    }
}

/// The escrow address follows the format as outlined in Cosmos SDK's ADR 028:
/// <https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-028-public-key-addresses.md/>
/// except that the `Hasher` function mandated by the `CryptoSpec` trait in the
/// rollup implementation.
pub fn compute_escrow_address<S: Spec>(port_id: &PortId, channel_id: &ChannelId) -> S::Address {
    let escrow_account_bytes: [u8; 32] = {
        let mut hasher = <S::CryptoSpec as CryptoSpec>::Hasher::new();
        hasher.update(VERSION);
        hasher.update([0]);
        hasher.update(format!("{port_id}/{channel_id}"));

        let hash = hasher.finalize();
        *hash.as_ref()
    };

    escrow_account_bytes.into()
}
