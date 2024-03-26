use anyhow::Result;
use sov_modules_api::{Module, Spec, WorkingSet};

use super::IbcTransfer;

impl<S: Spec> IbcTransfer<S> {
    pub(crate) fn init_module(
        &self,
        _config: &<Self as Module>::Config,
        _working_set: &mut WorkingSet<S>,
    ) -> Result<()> {
        Ok(())
    }
}
