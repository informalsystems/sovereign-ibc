use std::sync::Arc;

use ibc::Signer;

use super::handle::{Handle, QueryService};

/// Holds the necessary fields for querying a mock Cosmos
/// chain endpoint.
#[derive(Clone)]
pub struct ChainContext<E: Handle> {
    /// Chain handle
    querier: Arc<E>,
    /// relayer address on the chain for sending messages
    signer: Signer,
}

impl<E: Handle> ChainContext<E> {
    pub fn new(querier: Arc<E>, signer: Signer) -> Self {
        Self { querier, signer }
    }

    pub fn querier(&self) -> &Arc<E> {
        &self.querier
    }

    pub fn signer(&self) -> &Signer {
        &self.signer
    }
}

impl<E> QueryService for ChainContext<E>
where
    E: Handle,
{
    type E = E;

    fn service(&self) -> &Arc<Self::E> {
        &self.querier
    }
}
