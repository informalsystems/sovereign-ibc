//! Contains the runtime implementation for the Sovereign SDK rollup.
mod config;
pub use config::*;
use sov_bank::Bank;
use sov_ibc::Ibc;
use sov_ibc_transfer::IbcTransfer;
use sov_modules_api::hooks::{FinalizeHook, SlotHooks};
use sov_modules_api::macros::DefaultRuntime;
use sov_modules_api::{
    AccessoryStateCheckpoint, DaSpec, DispatchCall, Genesis, MessageCodec, Spec, StateCheckpoint,
    VersionedStateReadWriter,
};
use sov_state::Storage;

#[derive(Genesis, DispatchCall, MessageCodec, DefaultRuntime, Clone)]
#[serialization(serde::Serialize, serde::Deserialize)]
#[serialization(borsh::BorshDeserialize, borsh::BorshSerialize)]
pub struct Runtime<S, Da>
where
    S: Spec,
    Da: DaSpec,
{
    pub bank: Bank<S>,
    pub ibc: Ibc<S, Da>,
    pub ibc_transfer: IbcTransfer<S>,
}

impl<S: Spec, Da: DaSpec> SlotHooks for Runtime<S, Da> {
    type Spec = S;

    fn begin_slot_hook(
        &self,
        _pre_state_root: &<<Self::Spec as Spec>::Storage as Storage>::Root,
        _working_set: &mut VersionedStateReadWriter<StateCheckpoint<Self::Spec>>,
    ) {
    }

    fn end_slot_hook(&self, _working_set: &mut StateCheckpoint<Self::Spec>) {}
}

impl<S: Spec, Da: DaSpec> FinalizeHook for Runtime<S, Da> {
    type Spec = S;

    fn finalize_hook(
        &self,
        _root_hash: &<<Self::Spec as Spec>::Storage as Storage>::Root,
        _accesorry_working_set: &mut AccessoryStateCheckpoint<Self::Spec>,
    ) {
    }
}
