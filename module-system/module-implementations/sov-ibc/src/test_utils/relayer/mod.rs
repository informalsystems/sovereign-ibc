use basecoin_store::impls::InMemoryStore;
use sov_modules_api::default_context::DefaultContext;

use self::relay::MockRelayer;
use super::cosmos::app::MockCosmosChain;
use super::sovereign::app::TestApp;

pub mod context;
pub mod handle;
pub mod relay;

/// Concrete type for the relayer between a mock sovereign chain and a mock
pub type Relayer<'a> = MockRelayer<TestApp<'a, DefaultContext>, MockCosmosChain<InMemoryStore>>;
