//! Contains the runtime implementation for the Sovereign SDK rollup.
mod config;
pub use config::*;
use sov_bank::Bank;
use sov_ibc::Ibc;
use sov_ibc_transfer::IbcTransfer;
use sov_modules_api::hooks::{FinalizeHook, SlotHooks};
use sov_modules_api::macros::DefaultRuntime;
use sov_modules_api::{
    AccessoryWorkingSet, Context, DaSpec, DispatchCall, Genesis, MessageCodec, Spec,
    VersionedWorkingSet, WorkingSet,
};
use sov_state::Storage;

#[derive(Genesis, DispatchCall, MessageCodec, DefaultRuntime, Clone)]
#[serialization(serde::Serialize, serde::Deserialize)]
#[serialization(borsh::BorshDeserialize, borsh::BorshSerialize)]
pub struct Runtime<C, Da>
where
    C: Context,
    Da: DaSpec,
{
    pub bank: Bank<C>,
    pub ibc: Ibc<C, Da>,
    pub ibc_transfer: IbcTransfer<C>,
}

impl<C: Context, Da: DaSpec> SlotHooks for Runtime<C, Da> {
    type Context = C;

    fn begin_slot_hook(
        &self,
        _pre_state_root: &<<Self::Context as Spec>::Storage as Storage>::Root,
        _working_set: &mut VersionedWorkingSet<Self::Context>,
    ) {
    }

    fn end_slot_hook(&self, _working_set: &mut WorkingSet<C>) {}
}

impl<C: Context, Da: DaSpec> FinalizeHook for Runtime<C, Da> {
    type Context = C;

    fn finalize_hook(
        &self,
        _root_hash: &<<Self::Context as Spec>::Storage as Storage>::Root,
        _accesorry_working_set: &mut AccessoryWorkingSet<Self::Context>,
    ) {
    }
}
