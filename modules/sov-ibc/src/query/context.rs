use ibc::core::ics03_connection::connection::IdentifiedConnectionEnd;
use ibc::core::ics04_channel::channel::IdentifiedChannelEnd;
use ibc::core::ics04_channel::packet::Sequence;
use ibc::core::ics24_host::identifier::{ClientId, ConnectionId};
use ibc::core::ics24_host::path::{AckPath, ChannelEndPath, CommitmentPath, Path};
use ibc::core::{ContextError, ValidationContext};
use ibc::Height;
use ibc_query::core::context::{ProvableContext, QueryContext};
use sov_modules_api::{Context, DaSpec};

use crate::context::IbcContext;

impl<'a, C, Da> ProvableContext for IbcContext<'a, C, Da>
where
    C: Context,
    Da: DaSpec,
{
    /// TODO: Should figure out how can access the proof from the context
    fn get_proof(&self, _height: Height, _path: &Path) -> Option<Vec<u8>> {
        Some(vec![])
    }
}

impl<'a, C, Da> QueryContext for IbcContext<'a, C, Da>
where
    C: Context,
    Da: DaSpec,
{
    fn client_states(
        &self,
    ) -> Result<Vec<(ClientId, <Self as ValidationContext>::AnyClientState)>, ContextError> {
        unimplemented!()
    }

    fn consensus_states(
        &self,
        client_id: &ClientId,
    ) -> Result<
        Vec<(Height, <Self as ValidationContext>::AnyConsensusState)>,
        ibc::core::ContextError,
    > {
        unimplemented!()
    }

    fn consensus_state_heights(&self, client_id: &ClientId) -> Result<Vec<Height>, ContextError> {
        unimplemented!()
    }

    fn connection_ends(&self) -> Result<Vec<IdentifiedConnectionEnd>, ContextError> {
        unimplemented!()
    }

    fn client_connection_ends(
        &self,
        client_id: &ClientId,
    ) -> Result<Vec<ConnectionId>, ContextError> {
        unimplemented!()
    }

    fn channel_ends(&self) -> Result<Vec<IdentifiedChannelEnd>, ContextError> {
        unimplemented!()
    }

    fn packet_commitments(
        &self,
        channel_end_path: &ChannelEndPath,
    ) -> Result<Vec<CommitmentPath>, ContextError> {
        unimplemented!()
    }

    fn packet_acknowledgements(
        &self,
        channel_end_path: &ChannelEndPath,
        sequences: impl ExactSizeIterator<Item = Sequence>,
    ) -> Result<Vec<AckPath>, ContextError> {
        unimplemented!()
    }

    fn unreceived_packets(
        &self,
        channel_end_path: &ChannelEndPath,
        sequences: impl ExactSizeIterator<Item = Sequence>,
    ) -> Result<Vec<Sequence>, ContextError> {
        unimplemented!()
    }

    fn unreceived_acks(
        &self,
        channel_end_path: &ChannelEndPath,
        sequences: impl ExactSizeIterator<Item = Sequence>,
    ) -> Result<Vec<ibc::core::ics04_channel::packet::Sequence>, ContextError> {
        unimplemented!()
    }
}
