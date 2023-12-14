//! Defines a builder structure for instantiating a mock Cosmos chain
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;

use basecoin_store::context::ProvableStore;
use ibc_core::host::types::identifiers::ChainId;
use tendermint_testgen::Validator;
use tokio::runtime::Runtime;

use super::app::MockCosmosChain;

pub struct CosmosBuilder {
    pub runtime: Arc<Runtime>,
    pub chain_id: ChainId,
    pub validators: Vec<Validator>,
}

impl Default for CosmosBuilder {
    fn default() -> Self {
        Self::new(
            Arc::new(Runtime::new().unwrap()),
            ChainId::from_str("mock-cosmos-chain-0").expect("never fails"),
            vec![
                Validator::new("1").voting_power(40),
                Validator::new("2").voting_power(30),
                Validator::new("3").voting_power(30),
            ],
        )
    }
}

impl CosmosBuilder {
    pub fn new(runtime: Arc<Runtime>, chain_id: ChainId, validators: Vec<Validator>) -> Self {
        Self {
            runtime,
            chain_id,
            validators,
        }
    }

    pub fn build<S>(&mut self, store: S) -> MockCosmosChain<S>
    where
        S: ProvableStore + Debug + Default,
    {
        MockCosmosChain::new(
            self.runtime.clone(),
            self.chain_id.clone(),
            self.validators.clone(),
            store,
        )
    }
}
