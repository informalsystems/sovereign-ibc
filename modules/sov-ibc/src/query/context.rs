use ibc_client_tendermint::types::client_type as tm_client_type;
use ibc_core::channel::types::channel::IdentifiedChannelEnd;
use ibc_core::channel::types::error::ChannelError;
use ibc_core::channel::types::packet::PacketState;
use ibc_core::client::types::Height;
use ibc_core::connection::types::error::ConnectionError;
use ibc_core::connection::types::IdentifiedConnectionEnd;
use ibc_core::handler::types::error::ContextError;
use ibc_core::host::types::identifiers::{ChannelId, ClientId, ConnectionId, PortId, Sequence};
use ibc_core::host::types::path::{
    AckPath, ChannelEndPath, ClientConnectionPath, ClientConsensusStatePath, CommitmentPath, Path,
    ReceiptPath,
};
use ibc_core::host::ValidationContext;
use ibc_query::core::context::{ProvableContext, QueryContext};
use sov_modules_api::{Context, DaSpec, StateMapAccessor, StateValueAccessor, StateVecAccessor};

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
            let client_id = tm_client_type().build_client_id(i);
            let cs = self.client_state(&client_id)?;
            client_states.push((client_id, cs));
        }

        Ok(client_states)
    }

    fn consensus_states(
        &self,
        client_id: &ClientId,
    ) -> Result<Vec<(Height, <Self as ValidationContext>::AnyConsensusState)>, ContextError> {
        let update_heights: Vec<Height> = self
            .ibc
            .client_update_heights_vec
            .iter(*self.working_set.borrow_mut())
            .collect();

        let mut consesnsus_states = Vec::new();

        for height in update_heights {
            let cs = self.consensus_state(&ClientConsensusStatePath::new(
                client_id.clone(),
                height.revision_number(),
                height.revision_height(),
            ))?;
            consesnsus_states.push((height, cs));
        }

        Ok(consesnsus_states)
    }

    fn consensus_state_heights(&self, client_id: &ClientId) -> Result<Vec<Height>, ContextError> {
        let heights: Vec<Height> = self
            .ibc
            .client_update_heights_vec
            .iter(*self.working_set.borrow_mut())
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
    ) -> Result<Vec<PacketState>, ContextError> {
        self.ibc
            .packet_commitment_vec
            .iter(*self.working_set.borrow_mut())
            .filter(|commitment_path| {
                &ChannelEndPath::new(&commitment_path.port_id, &commitment_path.channel_id)
                    == channel_end_path
            })
            .map(|commitment_path| {
                self.get_packet_commitment(&commitment_path)
                    .map(|packet| PacketState {
                        seq: commitment_path.sequence,
                        port_id: commitment_path.port_id,
                        chan_id: commitment_path.channel_id,
                        data: packet.as_ref().into(),
                    })
            })
            .collect::<Result<Vec<_>, _>>()
    }

    fn packet_acknowledgements(
        &self,
        channel_end_path: &ChannelEndPath,
        sequences: impl ExactSizeIterator<Item = Sequence>,
    ) -> Result<Vec<PacketState>, ContextError> {
        let collected_paths: Vec<_> = if sequences.len() == 0 {
            self.ibc
                .packet_ack_vec
                .iter(*self.working_set.borrow_mut())
                .filter(|ack_path| {
                    &ChannelEndPath::new(&ack_path.port_id, &ack_path.channel_id)
                        == channel_end_path
                })
                .collect()
        } else {
            sequences
                .map(|seq| AckPath::new(&channel_end_path.0, &channel_end_path.1, seq))
                .collect()
        };

        collected_paths
            .into_iter()
            .map(|ack_path| {
                self.get_packet_acknowledgement(&ack_path)
                    .map(|packet| PacketState {
                        seq: ack_path.sequence,
                        port_id: ack_path.port_id,
                        chan_id: ack_path.channel_id,
                        data: packet.as_ref().into(),
                    })
            })
            .collect::<Result<Vec<_>, _>>()
    }

    fn unreceived_packets(
        &self,
        channel_end_path: &ChannelEndPath,
        sequences: impl ExactSizeIterator<Item = Sequence>,
    ) -> Result<Vec<Sequence>, ContextError> {
        Ok(sequences
            .map(|seq| ReceiptPath::new(&channel_end_path.0, &channel_end_path.1, seq))
            .filter(|receipt_path| self.get_packet_receipt(receipt_path).is_err())
            .map(|commitment_path| commitment_path.sequence)
            .collect())
    }

    fn unreceived_acks(
        &self,
        channel_end_path: &ChannelEndPath,
        sequences: impl ExactSizeIterator<Item = Sequence>,
    ) -> Result<Vec<Sequence>, ContextError> {
        let collected_paths: Vec<_> = if sequences.len() == 0 {
            self.ibc
                .packet_commitment_vec
                .iter(*self.working_set.borrow_mut())
                .filter(|commitment_path| {
                    &ChannelEndPath::new(&commitment_path.port_id, &commitment_path.channel_id)
                        == channel_end_path
                })
                .collect()
        } else {
            sequences
                .map(|seq| CommitmentPath::new(&channel_end_path.0, &channel_end_path.1, seq))
                .collect()
        };

        Ok(collected_paths
            .into_iter()
            .filter(|commitment_path| {
                self.ibc
                    .packet_commitment_map
                    .get(commitment_path, *self.working_set.borrow_mut())
                    .is_some()
            })
            .map(|commitment_path| commitment_path.sequence)
            .collect())
    }
}
