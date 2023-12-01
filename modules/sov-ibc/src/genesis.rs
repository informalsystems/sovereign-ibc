use anyhow::Result;
use sov_modules_api::{Context, DaSpec, Module, StateValueAccessor, WorkingSet};

use crate::Ibc;

impl<C: Context, Da: DaSpec> Ibc<C, Da> {
    pub(crate) fn init_module(
        &self,
        _config: &<Self as Module>::Config,
        working_set: &mut WorkingSet<C>,
    ) -> Result<()> {
        self.client_counter.set(&0, working_set);
        self.connection_counter.set(&0, working_set);
        self.channel_counter.set(&0, working_set);

        Ok(())
    }
}
