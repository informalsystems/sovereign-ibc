use sov_bank::Bank;
use sov_chain_state::ChainState;
use sov_modules_api::hooks::{FinalizeHook, SlotHooks};
use sov_modules_api::{AccessoryWorkingSet, Context, DaSpec, Module, Spec, WorkingSet};
use sov_state::Storage;

use super::config::TestConfig;
use crate::Ibc;

#[derive()]
pub struct TestRuntime<C, Da>
where
    C: Context,
    Da: DaSpec,
{
    pub chain_state: ChainState<C, Da>,
    pub bank: Bank<C>,
    pub ibc: Ibc<C, Da>,
}

impl<C, Da> Default for TestRuntime<C, Da>
where
    C: Context,
    Da: DaSpec,
{
    fn default() -> Self {
        Self {
            chain_state: ChainState::default(),
            bank: Bank::default(),
            ibc: Ibc::default(),
        }
    }
}

impl<C: Context, Da: DaSpec> TestRuntime<C, Da> {
    pub fn genesis(&mut self, cfg: &TestConfig<C>, working_set: &mut WorkingSet<C>) {
        self.chain_state
            .genesis(&cfg.chain_state_config, working_set)
            .unwrap();

        self.bank.genesis(&cfg.bank_config, working_set).unwrap();

        self.ibc.genesis(&cfg.ibc_config, working_set).unwrap();
    }
}

impl<C: Context, Da: DaSpec> SlotHooks<Da> for TestRuntime<C, Da> {
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

impl<C: Context, Da: DaSpec> FinalizeHook<Da> for TestRuntime<C, Da> {
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
