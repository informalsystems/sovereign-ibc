use borsh::BorshSerialize;
use ibc_client_tendermint::types::client_type as tm_client_type;
use ibc_core::channel::types::channel::{ChannelEnd, IdentifiedChannelEnd};
use ibc_core::channel::types::commitment::{AcknowledgementCommitment, PacketCommitment};
use ibc_core::channel::types::error::ChannelError;
use ibc_core::channel::types::packet::{PacketState, Receipt};
use ibc_core::client::context::ClientValidationContext;
use ibc_core::client::types::Height;
use ibc_core::connection::types::error::ConnectionError;
use ibc_core::connection::types::{ConnectionEnd, IdentifiedConnectionEnd};
use ibc_core::handler::types::error::ContextError;
use ibc_core::host::types::identifiers::{ChannelId, ClientId, ConnectionId, PortId, Sequence};
use ibc_core::host::types::path::{
    AckPath, ChannelEndPath, ClientConnectionPath, ClientConsensusStatePath, CommitmentPath,
    ConnectionPath, Path, ReceiptPath, SeqAckPath, SeqRecvPath, SeqSendPath, UpgradeClientPath,
};
use ibc_core::host::{ClientStateRef, ConsensusStateRef, ValidationContext};
use ibc_query::core::context::{ProvableContext, QueryContext};
use sov_celestia_client::client_state::ClientState as HostClientState;
use sov_celestia_client::consensus_state::ConsensusState as HostConsensusState;
use sov_modules_api::Spec;

use crate::context::IbcContext;
use crate::helpers::StorageValue;

impl<'a, S> IbcContext<'a, S>
where
    S: Spec,
{
    pub fn dyn_client_state<SV>(&self, client_id: &ClientId) -> SV::Output
    where
        SV: StorageValue<ClientStateRef<Self>>,
    {
        SV::value_at_key(
            client_id,
            &self.ibc.client_state_map,
            *self.working_set.borrow_mut(),
        )
    }

    pub fn dyn_client_consensus_state<SV>(
        &self,
        client_id: &ClientId,
        revision_number: u64,
        revision_height: u64,
    ) -> SV::Output
    where
        SV: StorageValue<ConsensusStateRef<Self>>,
    {
        let client_consensus_state_path =
            &ClientConsensusStatePath::new(client_id.clone(), revision_number, revision_height);

        SV::value_at_key(
            client_consensus_state_path,
            &self.ibc.consensus_state_map,
            *self.working_set.borrow_mut(),
        )
    }

    pub fn dyn_upgraded_client_state<SV>(&self, height: u64) -> SV::Output
    where
        SV: StorageValue<HostClientState>,
    {
        let upgrade_client_path = &UpgradeClientPath::UpgradedClientState(height);

        SV::value_at_key(
            upgrade_client_path,
            &self.ibc.upgraded_client_state_map,
            *self.working_set.borrow_mut(),
        )
    }

    pub fn dyn_upgraded_consensus_state<SV>(&self, height: u64) -> SV::Output
    where
        SV: StorageValue<HostConsensusState>,
    {
        let upgrade_client_consensus_path =
            &UpgradeClientPath::UpgradedClientConsensusState(height);

        SV::value_at_key(
            upgrade_client_consensus_path,
            &self.ibc.upgraded_consensus_state_map,
            *self.working_set.borrow_mut(),
        )
    }

    pub fn dyn_connection_end<SV>(&self, connection_id: &ConnectionId) -> SV::Output
    where
        SV: StorageValue<ConnectionEnd>,
    {
        let connection_path = &ConnectionPath::new(connection_id);

        SV::value_at_key(
            connection_path,
            &self.ibc.connection_end_map,
            *self.working_set.borrow_mut(),
        )
    }

    pub fn dyn_client_connections<SV>(&self, client_id: &ClientId) -> SV::Output
    where
        SV: StorageValue<Vec<ConnectionId>>,
    {
        let client_connection_path = &ClientConnectionPath::new(client_id.clone());

        SV::value_at_key(
            client_connection_path,
            &self.ibc.client_connections_map,
            *self.working_set.borrow_mut(),
        )
    }

    pub fn dyn_channel_end<SV>(&self, port_id: &PortId, channel_id: &ChannelId) -> SV::Output
    where
        SV: StorageValue<ChannelEnd>,
    {
        let channel_end_path = &ChannelEndPath::new(port_id, channel_id);

        SV::value_at_key(
            channel_end_path,
            &self.ibc.channel_end_map,
            *self.working_set.borrow_mut(),
        )
    }

    pub fn dyn_send_sequence<SV>(&self, port_id: &PortId, channel_id: &ChannelId) -> SV::Output
    where
        SV: StorageValue<Sequence>,
    {
        let seq_send_path = &SeqSendPath::new(port_id, channel_id);

        SV::value_at_key(
            seq_send_path,
            &self.ibc.send_sequence_map,
            *self.working_set.borrow_mut(),
        )
    }

    pub fn dyn_recv_sequence<SV>(&self, port_id: &PortId, channel_id: &ChannelId) -> SV::Output
    where
        SV: StorageValue<Sequence>,
    {
        let seq_recv_path = &SeqRecvPath::new(port_id, channel_id);

        SV::value_at_key(
            seq_recv_path,
            &self.ibc.recv_sequence_map,
            *self.working_set.borrow_mut(),
        )
    }

    pub fn dyn_ack_sequence<SV>(&self, port_id: &PortId, channel_id: &ChannelId) -> SV::Output
    where
        SV: StorageValue<Sequence>,
    {
        let seq_ack_path = &SeqAckPath::new(port_id, channel_id);

        SV::value_at_key(
            seq_ack_path,
            &self.ibc.ack_sequence_map,
            *self.working_set.borrow_mut(),
        )
    }

    pub fn dyn_packet_commitment<SV>(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: Sequence,
    ) -> SV::Output
    where
        SV: StorageValue<PacketCommitment>,
    {
        let commitment_path = &CommitmentPath::new(port_id, channel_id, sequence);

        SV::value_at_key(
            commitment_path,
            &self.ibc.packet_commitment_map,
            *self.working_set.borrow_mut(),
        )
    }

    pub fn dyn_packet_receipt<SV>(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: Sequence,
    ) -> SV::Output
    where
        SV: StorageValue<Receipt>,
    {
        let receipt_path = &ReceiptPath::new(port_id, channel_id, sequence);

        SV::value_at_key(
            receipt_path,
            &self.ibc.packet_receipt_map,
            *self.working_set.borrow_mut(),
        )
    }

    pub fn dyn_packet_acknowledgement<SV>(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: Sequence,
    ) -> SV::Output
    where
        SV: StorageValue<AcknowledgementCommitment>,
    {
        let ack_path = &AckPath::new(port_id, channel_id, sequence);

        SV::value_at_key(
            ack_path,
            &self.ibc.packet_ack_map,
            *self.working_set.borrow_mut(),
        )
    }
}

impl<'a, S> ProvableContext for IbcContext<'a, S>
where
    S: Spec,
{
    // NOTE: This is not efficient if used with a separate `get` call which returns only values.
    // Because `get_with_proof` will retrieve proof along with the value anyway.
    // Currently there is no way of retrieving only the proof.
    fn get_proof(&self, height: Height, path: &Path) -> Option<Vec<u8>> {
        let mut archival_working_set = self
            .working_set
            .borrow()
            .get_archival_at(height.revision_height());

        match path {
            Path::ClientState(client_state_path) => self
                .ibc
                .client_state_map
                .get_with_proof(&client_state_path.0, &mut archival_working_set),
            Path::ClientConsensusState(client_consensus_state_path) => self
                .ibc
                .consensus_state_map
                .get_with_proof(client_consensus_state_path, &mut archival_working_set),
            Path::Connection(connection_path) => self
                .ibc
                .connection_end_map
                .get_with_proof(connection_path, &mut archival_working_set),
            Path::ClientConnection(client_connection_path) => self
                .ibc
                .client_connections_map
                .get_with_proof(client_connection_path, &mut archival_working_set),
            Path::ChannelEnd(channel_end_path) => self
                .ibc
                .channel_end_map
                .get_with_proof(channel_end_path, &mut archival_working_set),
            Path::SeqSend(seq_send_path) => self
                .ibc
                .send_sequence_map
                .get_with_proof(seq_send_path, &mut archival_working_set),
            Path::SeqRecv(seq_recv_path) => self
                .ibc
                .recv_sequence_map
                .get_with_proof(seq_recv_path, &mut archival_working_set),
            Path::Commitment(commitment_path) => self
                .ibc
                .packet_commitment_map
                .get_with_proof(commitment_path, &mut archival_working_set),
            Path::Ack(ack_path) => self
                .ibc
                .packet_ack_map
                .get_with_proof(ack_path, &mut archival_working_set),
            Path::Receipt(receipt_path) => self
                .ibc
                .packet_receipt_map
                .get_with_proof(receipt_path, &mut archival_working_set),
            // not required in ibc-core; but still implemented
            Path::NextClientSequence(_) => self
                .ibc
                .client_counter
                .get_with_proof(&mut archival_working_set),
            Path::NextConnectionSequence(_) => self
                .ibc
                .connection_counter
                .get_with_proof(&mut archival_working_set),
            Path::NextChannelSequence(_) => self
                .ibc
                .channel_counter
                .get_with_proof(&mut archival_working_set),
            Path::UpgradeClient(upgrade_client_path) => self
                .ibc
                .upgraded_client_state_map
                .get_with_proof(upgrade_client_path, &mut archival_working_set),
            Path::SeqAck(seq_ack_path) => self
                .ibc
                .ack_sequence_map
                .get_with_proof(seq_ack_path, &mut archival_working_set),
            // not required, also not implemented; so `None` is returned
            Path::ClientUpdateTime(_) | Path::ClientUpdateHeight(_) | Path::Ports(_) => None?,
        }
        .try_to_vec()
        .ok()
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
