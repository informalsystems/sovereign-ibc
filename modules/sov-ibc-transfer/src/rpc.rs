//! Defines JSON RPC methods exposed by the ibc transfer module
use jsonrpsee::core::RpcResult;
use jsonrpsee::types::ErrorObjectOwned;
use sov_bank::TokenId;
use sov_modules_api::macros::rpc_gen;
use sov_modules_api::{Spec, WorkingSet};

use super::IbcTransfer;

#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
pub struct EscrowedTokenResponse {
    pub token_id: TokenId,
}

#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
pub struct MintedTokenResponse {
    pub token_id: TokenId,
}

#[rpc_gen(client, server, namespace = "transfer")]
impl<S> IbcTransfer<S>
where
    S: Spec,
{
    #[rpc_method(name = "escrowedToken")]
    pub fn escrowed_token(
        &self,
        token_denom: String,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<EscrowedTokenResponse> {
        let token_id =
            self.escrowed_tokens
                .get(&token_denom, working_set)
                .ok_or(ErrorObjectOwned::owned(
                    jsonrpsee::types::error::UNKNOWN_ERROR_CODE,
                    format!("No escrowed token found for denom {token_denom}"),
                    None::<String>,
                ))?;

        Ok(EscrowedTokenResponse { token_id })
    }

    #[rpc_method(name = "mintedToken")]
    pub fn minted_token(
        &self,
        token_denom: String,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<MintedTokenResponse> {
        let token_id =
            self.minted_tokens
                .get(&token_denom, working_set)
                .ok_or(ErrorObjectOwned::owned(
                    jsonrpsee::types::error::UNKNOWN_ERROR_CODE,
                    format!("No minted token found for denom {token_denom}"),
                    None::<String>,
                ))?;

        Ok(MintedTokenResponse { token_id })
    }
}
