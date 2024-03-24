use anyhow::Result;
use sov_modules_api::{DaSpec, Module, Spec, WorkingSet};

use crate::Ibc;

impl<S: Spec, Da: DaSpec> Ibc<S, Da> {
    pub(crate) fn init_module(
        &self,
        _config: &<Self as Module>::Config,
        working_set: &mut WorkingSet<S>,
    ) -> Result<()> {
        self.client_counter.set(&0, working_set);
        self.connection_counter.set(&0, working_set);
        self.channel_counter.set(&0, working_set);

        Ok(())
    }
}
