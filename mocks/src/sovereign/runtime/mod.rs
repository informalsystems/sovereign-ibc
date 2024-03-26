//! Contains the runtime implementation for the Sovereign SDK rollup.
mod config;
pub use config::*;
use sov_bank::Bank;
use sov_ibc::Ibc;
use sov_ibc_transfer::IbcTransfer;
use sov_modules_api::macros::DefaultRuntime;
use sov_modules_api::{DispatchCall, Genesis, MessageCodec, Spec};

#[derive(Genesis, DispatchCall, MessageCodec, DefaultRuntime, Clone)]
#[serialization(serde::Serialize, serde::Deserialize)]
#[serialization(borsh::BorshDeserialize, borsh::BorshSerialize)]
pub struct Runtime<S>
where
    S: Spec,
{
    pub bank: Bank<S>,
    pub ibc: Ibc<S>,
    pub ibc_transfer: IbcTransfer<S>,
}
