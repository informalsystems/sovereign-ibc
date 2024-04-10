//! Defines JSON RPC methods exposed by the ibc transfer module
use jsonrpsee::core::RpcResult;
use jsonrpsee::types::ErrorObjectOwned;
use sov_bank::TokenId;
use sov_modules_api::macros::rpc_gen;
use sov_modules_api::{Spec, WorkingSet};

use super::IbcTransfer;

#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
pub struct MintedTokenResponse {
    pub token_name: String,
    pub token_id: TokenId,
}

#[rpc_gen(client, server, namespace = "transfer")]
impl<S> IbcTransfer<S>
where
    S: Spec,
{
    #[rpc_method(name = "mintedTokenName")]
    pub fn escrowed_token(
        &self,
        token_id: TokenId,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<MintedTokenResponse> {
        let token_name = self
            .minted_token_id_to_name
            .get(&token_id, working_set)
            .ok_or(to_jsonrpsee_error(format!(
                "No IBC-minted token found for ID: '{token_id}'"
            )))?;

        Ok(MintedTokenResponse {
            token_name,
            token_id,
        })
    }

    #[rpc_method(name = "mintedTokenId")]
    pub fn minted_token(
        &self,
        token_name: String,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<MintedTokenResponse> {
        let token_id = self
            .minted_token_name_to_id
            .get(&token_name, working_set)
            .ok_or(to_jsonrpsee_error(format!(
                "No IBC-minted token found for denom: '{token_name}'"
            )))?;

        Ok(MintedTokenResponse {
            token_name,
            token_id,
        })
    }
}

/// Creates an jsonrpsee error object
pub fn to_jsonrpsee_error(err: impl ToString) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(
        jsonrpsee::types::error::UNKNOWN_ERROR_CODE,
        err.to_string(),
        None::<String>,
    )
}
