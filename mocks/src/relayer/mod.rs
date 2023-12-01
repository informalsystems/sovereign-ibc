pub mod context;
pub mod handle;
#[cfg(feature = "native")]
pub mod relay;

#[cfg(feature = "native")]
pub mod relayer_type {
    use basecoin_store::impls::InMemoryStore;
    use sov_mock_da::MockDaSpec;
    use sov_modules_api::default_context::DefaultContext;

    use super::relay::MockRelayer;
    use crate::cosmos::app::MockCosmosChain;
    use crate::sovereign::app::TestApp;

    /// Concrete type for the relayer between a mock sovereign chain and a mock
    pub type Relayer<'ws> =
        MockRelayer<TestApp<'ws, DefaultContext, MockDaSpec>, MockCosmosChain<InMemoryStore>>;
}

#[cfg(feature = "native")]
pub use relayer_type::*;
