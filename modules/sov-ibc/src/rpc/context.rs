use borsh::BorshSerialize;
use ibc_client_tendermint::types::client_type as tm_client_type;
use ibc_core::channel::types::channel::IdentifiedChannelEnd;
use ibc_core::channel::types::error::ChannelError;
use ibc_core::channel::types::packet::PacketState;
use ibc_core::client::context::ClientValidationContext;
use ibc_core::client::types::Height;
use ibc_core::connection::types::error::ConnectionError;
use ibc_core::connection::types::IdentifiedConnectionEnd;
use ibc_core::handler::types::error::ContextError;
use ibc_core::host::types::identifiers::{ChannelId, ClientId, ConnectionId, PortId, Sequence};
use ibc_core::host::types::path::{
    AckPath, ChannelEndPath, ClientConnectionPath, ClientConsensusStatePath, CommitmentPath, Path,
    ReceiptPath,
};
use ibc_core::host::{ClientStateRef, ConsensusStateRef, ValidationContext};
use ibc_query::core::context::{ProvableContext, QueryContext};
use sov_modules_api::Spec;

use crate::context::IbcContext;

impl<'a, S> ProvableContext for IbcContext<'a, S>
where
    S: Spec,
{
    /// TODO: Should figure out how can access the proof from the context
    fn get_proof(&self, height: Height, path: &Path) -> Option<Vec<u8>> {
        let result = match path {
            Path::ClientState(client_state_path) => {
                let mut archival_working_set = self
                    .working_set
                    .borrow()
                    .get_archival_at(height.revision_height());

                self.ibc
                    .client_state_map
                    .get_with_proof(&client_state_path.0, &mut archival_working_set)
                    .proof
                    .try_to_vec()
            }
            Path::ClientConsensusState(client_consensus_state_path) => {
                let mut archival_working_set = self
                    .working_set
                    .borrow()
                    .get_archival_at(height.revision_height());

                self.ibc
                    .consensus_state_map
                    .get_with_proof(client_consensus_state_path, &mut archival_working_set)
                    .proof
                    .try_to_vec()
            }
            Path::ClientConnection(client_connection_path) => {
                let mut archival_working_set = self
                    .working_set
                    .borrow()
                    .get_archival_at(height.revision_height());

                self.ibc
                    .client_connections_map
                    .get_with_proof(client_connection_path, &mut archival_working_set)
                    .proof
                    .try_to_vec()
            }
            Path::Connection(connection_path) => {
                let mut archival_working_set = self
                    .working_set
                    .borrow()
                    .get_archival_at(height.revision_height());

                self.ibc
                    .connection_end_map
                    .get_with_proof(connection_path, &mut archival_working_set)
                    .proof
                    .try_to_vec()
            }
            Path::ChannelEnd(channel_end_path) => {
                let mut archival_working_set = self
                    .working_set
                    .borrow()
                    .get_archival_at(height.revision_height());

                self.ibc
                    .channel_end_map
                    .get_with_proof(channel_end_path, &mut archival_working_set)
                    .proof
                    .try_to_vec()
            }
            Path::SeqSend(seq_send_path) => {
                let mut archival_working_set = self
                    .working_set
                    .borrow()
                    .get_archival_at(height.revision_height());

                self.ibc
                    .send_sequence_map
                    .get_with_proof(seq_send_path, &mut archival_working_set)
                    .proof
                    .try_to_vec()
            }
            Path::SeqRecv(seq_recv_path) => {
                let mut archival_working_set = self
                    .working_set
                    .borrow()
                    .get_archival_at(height.revision_height());

                self.ibc
                    .recv_sequence_map
                    .get_with_proof(seq_recv_path, &mut archival_working_set)
                    .proof
                    .try_to_vec()
            }
            Path::Commitment(commitment_path) => {
                let mut archival_working_set = self
                    .working_set
                    .borrow()
                    .get_archival_at(height.revision_height());

                self.ibc
                    .packet_commitment_map
                    .get_with_proof(commitment_path, &mut archival_working_set)
                    .proof
                    .try_to_vec()
            }
            Path::Ack(ack_path) => {
                let mut archival_working_set = self
                    .working_set
                    .borrow()
                    .get_archival_at(height.revision_height());

                self.ibc
                    .packet_ack_map
                    .get_with_proof(ack_path, &mut archival_working_set)
                    .proof
                    .try_to_vec()
            }
            Path::Receipt(receipt_path) => {
                let mut archival_working_set = self
                    .working_set
                    .borrow()
                    .get_archival_at(height.revision_height());

                self.ibc
                    .packet_receipt_map
                    .get_with_proof(receipt_path, &mut archival_working_set)
                    .proof
                    .try_to_vec()
            }
            // not required; but should filled in for completeness
            Path::SeqAck(_) => todo!(),
            Path::UpgradeClient(_) => todo!(),
            Path::NextClientSequence(_) => todo!(),
            Path::NextConnectionSequence(_) => todo!(),
            Path::NextChannelSequence(_) => todo!(),
            Path::ClientUpdateTime(_) => todo!(),
            Path::ClientUpdateHeight(_) => todo!(),
            Path::Ports(_) => todo!(),
        };

        result.ok()
    }
}

impl<'a, S> QueryContext for IbcContext<'a, S>
where
    S: Spec,
{
    fn client_states(&self) -> Result<Vec<(ClientId, ClientStateRef<Self>)>, ContextError> {
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
    ) -> Result<Vec<(Height, ConsensusStateRef<Self>)>, ContextError> {
        let update_heights: Vec<Height> = self
            .ibc
            .client_update_heights_vec
            .iter(*self.working_set.borrow_mut())
            .collect();

        let mut consensus_states = Vec::new();

        for height in update_heights {
            let cs = self.get_client_validation_context().consensus_state(
                &ClientConsensusStatePath::new(
                    client_id.clone(),
                    height.revision_number(),
                    height.revision_height(),
                ),
            )?;
            consensus_states.push((height, cs));
        }

        Ok(consensus_states)
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
                &ClientConnectionPath::new(client_id.clone()),
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
