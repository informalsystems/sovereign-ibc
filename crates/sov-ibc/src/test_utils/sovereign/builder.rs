use ibc::core::ics24_host::identifier::ChainId;
use sov_bank::TokenConfig;
use sov_modules_api::default_context::DefaultContext;
use sov_modules_api::{Context, WorkingSet};
use sov_rollup_interface::mocks::MockDaSpec;
use sov_state::ProverStorage;

use super::app::TestApp;
use super::config::TestConfig;
use super::runtime::TestRuntime;

/// Defines a builder structure with default configurations and specs for
/// instantiating a mock Sovereign SDK application
pub struct DefaultBuilder {
    chain_id: ChainId,
    config: TestConfig<DefaultContext>,
    runtime: TestRuntime<DefaultContext, MockDaSpec>,
    working_set: WorkingSet<DefaultContext>,
}

impl Default for DefaultBuilder {
    fn default() -> Self {
        let chain_id = ChainId::new("ibc", 0).unwrap();

        let tmpdir = tempfile::tempdir().unwrap();

        let mut working_set = WorkingSet::new(ProverStorage::with_path(tmpdir.path()).unwrap());

        let cfg = TestConfig::default();

        let mut runtime = TestRuntime::default();

        runtime.genesis(&cfg, &mut working_set);

        Self::new(chain_id, cfg, runtime, working_set)
    }
}

impl DefaultBuilder {
    /// Creates a new test fixture builder with default storage spec
    pub fn new(
        chain_id: ChainId,
        config: TestConfig<DefaultContext>,
        runtime: TestRuntime<DefaultContext, MockDaSpec>,
        working_set: WorkingSet<DefaultContext>,
    ) -> Self {
        Self {
            chain_id,
            config,
            runtime,
            working_set,
        }
    }

    /// Returns list of tokens in the bank configuration
    pub fn get_tokens(&self) -> &Vec<TokenConfig<DefaultContext>> {
        &self.config.bank_config.tokens
    }

    /// Builds a test fixture with default configuration
    pub fn build<'a>(&'a mut self) -> TestApp<'a, DefaultContext, MockDaSpec> {
        let relayer_address = self.config.bank_config.tokens[0]
            .address_and_balances
            .last()
            .unwrap();

        TestApp::<'a, DefaultContext, MockDaSpec>::new(
            self.chain_id.clone(),
            DefaultContext::new(relayer_address.0),
            &self.runtime.ibc,
            &mut self.working_set,
        )
    }
}
