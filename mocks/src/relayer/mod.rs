pub mod context;
pub mod handle;
#[cfg(feature = "native")]
pub mod relay;

#[cfg(feature = "native")]
pub mod relayer_type {
    use basecoin_store::impls::InMemoryStore;
    use sov_mock_da::MockDaService;
    use sov_modules_api::default_context::DefaultContext;
    use sov_state::DefaultStorageSpec;

    use super::relay::MockRelayer;
    use crate::cosmos::MockCosmosChain;
    use crate::sovereign::MockRollup;

    /// Default concrete type for the relayer between the mock rollup and the
    /// mock Cosmos chain.
    pub type DefaultRelayer = MockRelayer<
        MockRollup<DefaultContext, MockDaService, DefaultStorageSpec>,
        MockCosmosChain<InMemoryStore>,
    >;

    /// Default concrete type for the mock rollup.
    pub type DefaultRollup = MockRollup<DefaultContext, MockDaService, DefaultStorageSpec>;
}

#[cfg(feature = "native")]
pub use relayer_type::*;
