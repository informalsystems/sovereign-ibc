//! Defines rpc queries exposed by the ibc transfer module
use jsonrpsee::core::RpcResult;
use jsonrpsee::types::ErrorObjectOwned;
use sov_modules_api::macros::rpc_gen;
use sov_modules_api::{Context, WorkingSet};

use super::Transfer;

#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
pub struct EscrowedTokenResponse<C: Context> {
    pub address: C::Address,
}

#[rpc_gen(client, server, namespace = "transfer")]
impl<C> Transfer<C>
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
}
