//! Defines JSON RPC methods exposed by the ibc transfer module
use jsonrpsee::core::RpcResult;
use jsonrpsee::types::ErrorObjectOwned;
use sov_bank::TokenId;
use sov_modules_api::macros::rpc_gen;
use sov_modules_api::{Spec, WorkingSet};

use super::IbcTransfer;

#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
pub struct MintedTokenResponse {
    pub token_id: TokenId,
}

#[rpc_gen(client, server, namespace = "transfer")]
impl<S> IbcTransfer<S>
where
    S: Spec,
{
    #[rpc_method(name = "mintedToken")]
    pub fn minted_token(
        &self,
        token_denom: String,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<MintedTokenResponse> {
        let token_id = self
            .minted_token_name_to_id
            .get(&token_denom, working_set)
            .ok_or(ErrorObjectOwned::owned(
                jsonrpsee::types::error::UNKNOWN_ERROR_CODE,
                format!("No minted token found for denom {token_denom}"),
                None::<String>,
            ))?;

        Ok(MintedTokenResponse { token_id })
    }
}
