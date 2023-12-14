use std::sync::Arc;

use ibc_core::primitives::Signer;

use super::handle::{Handle, QueryService};

/// Defines the chain context by which the relayer interacts with the chain.
#[derive(Clone)]
pub struct ChainContext<E: Handle> {
    /// Mimics the chain service for querying the chain state
    service: Arc<E>,
    /// relayer address to sign and submit messages
    signer: Signer,
}

impl<E: Handle> ChainContext<E> {
    pub fn new(service: Arc<E>, signer: Signer) -> Self {
        Self { service, signer }
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
        &self.service
    }
}
