//! Defines a builder structure for instantiating a mock Cosmos chain
use std::fmt::Debug;

use basecoin_store::context::ProvableStore;

use super::app::MockCosmosChain;
use super::MockTendermint;

pub struct CosmosBuilder<S>
where
    S: ProvableStore + Debug,
{
    core: MockTendermint,
    store: S,
}

impl<S> Default for CosmosBuilder<S>
where
    S: ProvableStore + Debug + Default,
{
    fn default() -> Self {
        Self::new(MockTendermint::builder().build(), S::default())
    }
}

impl<S> CosmosBuilder<S>
where
    S: ProvableStore + Debug,
{
    pub fn new(core: MockTendermint, store: S) -> Self {
        Self { core, store }
    }

    pub fn build(self) -> MockCosmosChain<S>
    where
        S: ProvableStore + Debug + Default,
    {
        MockCosmosChain::new(self.core, self.store)
    }
}
