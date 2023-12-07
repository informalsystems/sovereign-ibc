use sov_bank::Bank;
use sov_chain_state::ChainState;
use sov_ibc::Ibc;
use sov_ibc_transfer::IbcTransfer;
use sov_modules_api::hooks::{FinalizeHook, SlotHooks};
use sov_modules_api::macros::DefaultRuntime;
use sov_modules_api::{
    AccessoryWorkingSet, Context, DaSpec, DispatchCall, Genesis, MessageCodec, Spec, WorkingSet,
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
    pub chain_state: ChainState<C, Da>,
    pub bank: Bank<C>,
    pub ibc: Ibc<C, Da>,
    pub ibc_transfer: IbcTransfer<C>,
}

impl<C: Context, Da: DaSpec> SlotHooks<Da> for Runtime<C, Da> {
    type Context = C;

    fn begin_slot_hook(
        &self,
        slot_header: &Da::BlockHeader,
        validity_condition: &Da::ValidityCondition,
        pre_state_root: &<<Self::Context as Spec>::Storage as Storage>::Root,
        working_set: &mut WorkingSet<Self::Context>,
    ) {
        self.chain_state.begin_slot_hook(
            slot_header,
            validity_condition,
            pre_state_root,
            working_set,
        );
    }

    fn end_slot_hook(&self, working_set: &mut WorkingSet<C>) {
        self.chain_state.end_slot_hook(working_set);
    }
}

impl<C: Context, Da: DaSpec> FinalizeHook<Da> for Runtime<C, Da> {
    type Context = C;

    fn finalize_hook(
        &self,
        root_hash: &<<Self::Context as Spec>::Storage as Storage>::Root,
        accesorry_working_set: &mut AccessoryWorkingSet<Self::Context>,
    ) {
        self.chain_state
            .finalize_hook(root_hash, accesorry_working_set);
    }
}
