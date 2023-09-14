use ibc::core::ics24_host::identifier::ChainId;
use sov_bank::{Bank, BankConfig, TokenConfig};
use sov_modules_api::default_context::DefaultContext;
use sov_modules_api::{Context, Module};
use sov_state::{DefaultStorageSpec, ProverStorage, WorkingSet};

use super::app::TestApp;
use crate::test_utils::config::create_bank_config;
use crate::{ExampleModuleConfig, Ibc};

/// Defines a test fixture builder with default configurations and specs
pub struct Builder {
    chain_id: Option<ChainId>,
    working_set: WorkingSet<ProverStorage<DefaultStorageSpec>>,
    bank_config: BankConfig<DefaultContext>,
    ibc_config: ExampleModuleConfig,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// Creates a new test fixture builder with default storage spec
    pub fn new() -> Self {
        let tmpdir = tempfile::tempdir().unwrap();

        let working_set = WorkingSet::new(ProverStorage::with_path(tmpdir.path()).unwrap());

        let initial_balance = 1000;

        let address_count = 3;

        let bank_config = create_bank_config(address_count, initial_balance);

        Self {
            chain_id: None,
            working_set,
            bank_config,
            ibc_config: ExampleModuleConfig {},
        }
    }

    pub fn set_chain_id(&mut self, chain_id: ChainId) {
        self.chain_id = Some(chain_id);
    }

    pub fn set_bank_config(&mut self, bank_config: BankConfig<DefaultContext>) {
        self.bank_config = bank_config;
    }

    /// Returns list of tokens in the bank configuration
    pub fn get_tokens(&self) -> &Vec<TokenConfig<DefaultContext>> {
        &self.bank_config.tokens
    }

    /// Builds a test fixture with default configuration
    pub fn build(&mut self) -> TestApp<'_, DefaultContext> {
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

        let chain_id = self
            .chain_id
            .clone()
            .unwrap_or(ChainId::new("ibc", 0).unwrap());

        let sdk_ctx = DefaultContext::new(relayer_address.0);

        TestApp::<DefaultContext>::new(chain_id, sdk_ctx, bank, ibc, &mut self.working_set)
    }
}
