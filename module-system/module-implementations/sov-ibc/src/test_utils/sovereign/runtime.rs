use sov_bank::Bank;
use sov_chain_state::ChainState;
use sov_modules_api::hooks::SlotHooks;
use sov_modules_api::{Context, DaSpec, Module};
use sov_state::{AccessoryWorkingSet, WorkingSet};

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

impl<C, Da> TestRuntime<C, Da>
where
    C: Context,
    Da: DaSpec,
{
    pub fn genesis(&mut self, cfg: &TestConfig<C>, working_set: &mut WorkingSet<C::Storage>) {
        self.chain_state
            .genesis(&cfg.chain_state_config, working_set)
            .unwrap();

        self.bank.genesis(&cfg.bank_config, working_set).unwrap();

        self.ibc.genesis(&cfg.ibc_config, working_set).unwrap();
    }
}

impl<C, Da> SlotHooks<Da> for TestRuntime<C, Da>
where
    C: Context,
    Da: DaSpec,
{
    type Context = C;

    fn begin_slot_hook(
        &self,
        slot_header: &Da::BlockHeader,
        validity_condition: &Da::ValidityCondition,
        working_set: &mut WorkingSet<C::Storage>,
    ) {
        unimplemented!()
    }

    fn end_slot_hook(&self, working_set: &mut WorkingSet<C::Storage>) {
        unimplemented!()
    }

    fn finalize_slot_hook(
        &self,
        root_hash: [u8; 32],
        accesorry_working_set: &mut AccessoryWorkingSet<C::Storage>,
    ) {
        unimplemented!()
    }
}
