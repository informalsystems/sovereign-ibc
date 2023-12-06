//! Defines rpc queries exposed by the ibc transfer module
use jsonrpsee::core::RpcResult;
use jsonrpsee::types::ErrorObjectOwned;
use sov_modules_api::macros::rpc_gen;
use sov_modules_api::{Context, StateMapAccessor, WorkingSet};

use super::IbcTransfer;

#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
pub struct EscrowedTokenResponse<C: Context> {
    pub address: C::Address,
}

#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
pub struct MintedTokenResponse<C: Context> {
    pub address: C::Address,
}

#[rpc_gen(client, server, namespace = "transfer")]
impl<C> IbcTransfer<C>
where
    C: Context,
{
    #[rpc_method(name = "escrowedToken")]
    pub fn escrowed_token(
        &self,
        token_denom: String,
        working_set: &mut WorkingSet<C>,
    ) -> RpcResult<EscrowedTokenResponse<C>> {
        let token_address =
            self.escrowed_tokens
                .get(&token_denom, working_set)
                .ok_or(ErrorObjectOwned::owned(
                    jsonrpsee::types::error::UNKNOWN_ERROR_CODE,
                    format!("No escrowed token found for denom {}", token_denom),
                    None::<String>,
                ))?;

        Ok(EscrowedTokenResponse {
            address: token_address.clone(),
        })
    }

    #[rpc_method(name = "mintedToken")]
    pub fn minted_token(
        &self,
        token_denom: String,
        working_set: &mut WorkingSet<C>,
    ) -> RpcResult<MintedTokenResponse<C>> {
        let token_address =
            self.minted_tokens
                .get(&token_denom, working_set)
                .ok_or(ErrorObjectOwned::owned(
                    jsonrpsee::types::error::UNKNOWN_ERROR_CODE,
                    format!("No minted token found for denom {}", token_denom),
                    None::<String>,
                ))?;

        Ok(MintedTokenResponse {
            address: token_address.clone(),
        })
    }
}
