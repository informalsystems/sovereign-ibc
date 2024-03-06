#[cfg(feature = "native")]
mod builder;
mod context;
mod handle;
#[cfg(feature = "native")]
mod msgs;
mod relay;

#[cfg(feature = "native")]
pub use builder::*;
pub use context::*;
pub use handle::*;
pub use relay::*;

#[cfg(feature = "native")]
mod types {
    use basecoin_store::impls::InMemoryStore;

    use super::relay::MockRelayer;
    use crate::cosmos::MockCosmosChain;
    use crate::sovereign::MockRollup;

    /// Default concrete type for the relayer between the mock rollup and the
    /// mock Cosmos chain.
    pub type DefaultRelayer<S, Da, P> =
        MockRelayer<MockRollup<S, Da, P>, MockCosmosChain<InMemoryStore>>;
}

#[cfg(feature = "native")]
pub use types::*;
