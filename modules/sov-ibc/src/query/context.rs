use ibc::clients::ics07_tendermint::client_type as tm_client_type;
use ibc::core::ics02_client::error::ClientError;
use ibc::core::ics03_connection::connection::IdentifiedConnectionEnd;
use ibc::core::ics03_connection::error::ConnectionError;
use ibc::core::ics04_channel::channel::IdentifiedChannelEnd;
use ibc::core::ics04_channel::error::ChannelError;
use ibc::core::ics04_channel::packet::Sequence;
use ibc::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
use ibc::core::ics24_host::path::{
    AckPath, ChannelEndPath, ClientConnectionPath, ClientConsensusStatePath, CommitmentPath, Path,
};
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
        let client_counter = self
            .ibc
            .client_counter
            .get(*self.working_set.borrow_mut())
            .ok_or(ChannelError::Other {
                description: "Connection counter not found".to_string(),
            })?;

        let mut client_states = Vec::new();

        for i in 0..client_counter {
            // ibc-rs already only supports the Tendermint client
            let client_id =
                ClientId::new(tm_client_type(), i).map_err(ClientError::InvalidClientIdentifier)?;

            let cs = self.client_state(&client_id)?;
            client_states.push((client_id, cs));
        }

        Ok(client_states)
    }

    fn consensus_states(
        &self,
        client_id: &ClientId,
    ) -> Result<
        Vec<(Height, <Self as ValidationContext>::AnyConsensusState)>,
        ibc::core::ContextError,
    > {
        let update_heights: Vec<Height> = self
            .ibc
            .client_update_heights_vec
            .iter(*self.working_set.borrow_mut())
            .map(|h| h)
            .collect();

        let mut consesnsus_states = Vec::new();

        for height in update_heights {
            let cs = self.consensus_state(&ClientConsensusStatePath::new(client_id, &height))?;
            consesnsus_states.push((height, cs));
        }

        Ok(consesnsus_states)
    }

    fn consensus_state_heights(&self, client_id: &ClientId) -> Result<Vec<Height>, ContextError> {
        let heights: Vec<Height> = self
            .ibc
            .client_update_heights_vec
            .iter(*self.working_set.borrow_mut())
            .map(|h| h)
            .collect();

        Ok(heights)
    }

    fn connection_ends(&self) -> Result<Vec<IdentifiedConnectionEnd>, ContextError> {
        let conn_counter = self
            .ibc
            .connection_counter
            .get(*self.working_set.borrow_mut())
            .ok_or(ConnectionError::Other {
                description: "Connection counter not found".to_string(),
            })?;

        let mut conn_ends = Vec::new();

        for i in 0..conn_counter {
            let conn_id = ConnectionId::new(i);
            let conn_end = self.connection_end(&conn_id)?;
            conn_ends.push(IdentifiedConnectionEnd::new(conn_id, conn_end));
        }

        Ok(conn_ends)
    }

    fn client_connection_ends(
        &self,
        client_id: &ClientId,
    ) -> Result<Vec<ConnectionId>, ContextError> {
        let client_connections = self
            .ibc
            .client_connections_map
            .get(
                &ClientConnectionPath::new(client_id),
                *self.working_set.borrow_mut(),
            )
            .ok_or(ConnectionError::Other {
                description: "Client connections not found".to_string(),
            })?;

        Ok(client_connections)
    }

    fn channel_ends(&self) -> Result<Vec<IdentifiedChannelEnd>, ContextError> {
        let chan_counter = self
            .ibc
            .channel_counter
            .get(*self.working_set.borrow_mut())
            .ok_or(ChannelError::Other {
                description: "Connection counter not found".to_string(),
            })?;

        let mut chan_ends = Vec::new();

        for i in 0..chan_counter {
            // ibc-rs already only supports the Token Transfer application
            let port_id = PortId::transfer();
            let chan_id = ChannelId::new(i);
            let chan_end_path = ChannelEndPath::new(&port_id, &chan_id);
            let chan_end = self.channel_end(&chan_end_path)?;
            chan_ends.push(IdentifiedChannelEnd::new(port_id, chan_id, chan_end));
        }

        Ok(chan_ends)
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
    ) -> Result<Vec<Sequence>, ContextError> {
        unimplemented!()
    }
}
