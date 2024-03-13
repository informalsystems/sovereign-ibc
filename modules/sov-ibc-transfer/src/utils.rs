use ibc_app_transfer::types::VERSION;
use ibc_core::host::types::identifiers::{ChannelId, PortId};
use sov_modules_api::digest::Digest;
use sov_modules_api::{CryptoSpec, Spec};

/// The escrow address follows the format as outlined in Cosmos SDK's ADR 028:
/// https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-028-public-key-addresses.md
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
