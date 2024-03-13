use ibc_core::client::context::ClientValidationContext;
use ibc_core::client::types::Height;
use ibc_core::handler::types::error::ContextError;
use ibc_core::host::types::identifiers::ClientId;
use ibc_core::host::types::path::ClientConsensusStatePath;
use ibc_core::primitives::Timestamp;
use sov_celestia_client::context::{
    ConsensusStateConverter, ValidationContext as SovValidationContext,
};

use super::Context;
use crate::types::{ClientType, HeightTravel};

impl<'a, C: ClientType<'a>> SovValidationContext for Context<'a, C>
where
    <C as ClientType<'a>>::ConsensusState: ConsensusStateConverter,
{
    fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
        let time = self.env().block.time;

        let host_timestamp = Timestamp::from_nanoseconds(time.nanos()).expect("invalid timestamp");

        Ok(host_timestamp)
    }

    fn host_height(&self) -> Result<Height, ContextError> {
        let host_height = Height::new(0, self.env().block.height)?;

        Ok(host_height)
    }

    fn consensus_state_heights(&self, _client_id: &ClientId) -> Result<Vec<Height>, ContextError> {
        let heights = self.get_heights()?;

        Ok(heights)
    }
    fn next_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::ConsensusStateRef>, ContextError> {
        let next_height = self.get_adjacent_height(height, HeightTravel::Next)?;

        match next_height {
            Some(h) => {
                let cons_state_path = ClientConsensusStatePath::new(
                    client_id.clone(),
                    h.revision_number(),
                    h.revision_height(),
                );
                self.consensus_state(&cons_state_path).map(Some)
            }
            None => Ok(None),
        }
    }

    fn prev_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::ConsensusStateRef>, ContextError> {
        let prev_height = self.get_adjacent_height(height, HeightTravel::Prev)?;

        match prev_height {
            Some(prev_height) => {
                let cons_state_path = ClientConsensusStatePath::new(
                    client_id.clone(),
                    prev_height.revision_number(),
                    prev_height.revision_height(),
                );
                self.consensus_state(&cons_state_path).map(Some)
            }
            None => Ok(None),
        }
    }
}
