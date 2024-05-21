use anyhow::Result;
use sov_modules_api::{GenesisState, Module, Spec};

use crate::Ibc;

impl<S: Spec> Ibc<S> {
    pub(crate) fn init_module(
        &self,
        _config: &<Self as Module>::Config,
        working_set: &mut impl GenesisState<S>,
    ) -> Result<()> {
        self.client_counter.set(&0, working_set);
        self.connection_counter.set(&0, working_set);
        self.channel_counter.set(&0, working_set);

        Ok(())
    }
}
