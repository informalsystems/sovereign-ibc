use ibc::core::ics24_host::identifier::ChainId;
use sov_bank::{Bank, BankConfig, TokenConfig};
use sov_modules_api::default_context::DefaultContext;
use sov_modules_api::{Context, Module};
use sov_state::{DefaultStorageSpec, ProverStorage, WorkingSet};

use super::app::TestApp;
use super::config::create_bank_config;
use crate::{ExampleModuleConfig, Ibc};

/// Defines a test fixture builder with default configurations and specs
pub struct Builder {
    chain_id: ChainId,
    bank_config: BankConfig<DefaultContext>,
    ibc_config: ExampleModuleConfig,
    working_set: WorkingSet<ProverStorage<DefaultStorageSpec>>,
}

impl Default for Builder {
    fn default() -> Self {
        let chain_id = ChainId::new("ibc", 0).unwrap();

        let initial_balance = 1000;

        let address_count = 3;

        let bank_config = create_bank_config(address_count, initial_balance);

        let tmpdir = tempfile::tempdir().unwrap();

        let working_set = WorkingSet::new(ProverStorage::with_path(tmpdir.path()).unwrap());

        Self::new(chain_id, bank_config, ExampleModuleConfig {}, working_set)
    }
}

impl Builder {
    /// Creates a new test fixture builder with default storage spec
    pub fn new(
        chain_id: ChainId,
        bank_config: BankConfig<DefaultContext>,
        ibc_config: ExampleModuleConfig,
        working_set: WorkingSet<ProverStorage<DefaultStorageSpec>>,
    ) -> Self {
        Self {
            chain_id,
            working_set,
            bank_config,
            ibc_config: ExampleModuleConfig {},
        }
    }

    /// Returns list of tokens in the bank configuration
    pub fn get_tokens(&self) -> &Vec<TokenConfig<DefaultContext>> {
        &self.bank_config.tokens
    }

    /// Builds a test fixture with default configuration
    pub fn build<'a>(&'a mut self) -> TestApp<'a, DefaultContext> {
        // Initialize the bank module
        let bank = Bank::<DefaultContext>::default();
        bank.genesis(&self.bank_config, &mut self.working_set)
            .unwrap();

        // Initialize the ibc module
        let ibc = Ibc::<DefaultContext>::default();
        ibc.genesis(&self.ibc_config, &mut self.working_set)
            .unwrap();

        let relayer_address = self.bank_config.tokens[0]
            .address_and_balances
            .last()
            .unwrap();

        let sdk_ctx = DefaultContext::new(relayer_address.0);

        TestApp::<'a, DefaultContext>::new(
            self.chain_id.clone(),
            sdk_ctx,
            bank,
            ibc,
            &mut self.working_set,
        )
    }
}
