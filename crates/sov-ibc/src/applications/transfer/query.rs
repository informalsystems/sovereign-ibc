//! Defines rpc queries exposed by the ibc transfer module
use jsonrpsee::core::RpcResult;
use sov_modules_api::macros::rpc_gen;
use sov_modules_api::{Context, WorkingSet};

use super::Transfer;

#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
pub struct EscrowedTokenResponse<C: Context> {
    pub denom: String,
    pub address: C::Address,
}

#[rpc_gen(client, server, namespace = "transfer")]
impl<C> Transfer<C>
where
    C: Context,
{
    #[rpc_method(name = "escrowedTokens")]
    pub fn escrowed_tokens(
        &self,
        working_set: &mut WorkingSet<C>,
    ) -> RpcResult<EscrowedTokenResponse<C>> {
        unimplemented!()
    }
}
