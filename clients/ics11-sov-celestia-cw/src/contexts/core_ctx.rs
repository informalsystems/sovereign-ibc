use std::time::Duration;

use ibc::core::events::IbcEvent;
use ibc::core::ics02_client::error::ClientError;
use ibc::core::ics03_connection::connection::ConnectionEnd;
use ibc::core::ics04_channel::channel::ChannelEnd;
use ibc::core::ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment};
use ibc::core::ics04_channel::packet::{Receipt, Sequence};
use ibc::core::ics23_commitment::commitment::CommitmentPrefix;
use ibc::core::ics24_host::identifier::{ClientId, ConnectionId};
use ibc::core::ics24_host::path::{
    AckPath, ChannelEndPath, ClientConnectionPath, ClientConsensusStatePath, ClientStatePath,
    CommitmentPath, ConnectionPath, ReceiptPath, SeqAckPath, SeqRecvPath, SeqSendPath,
};
use ibc::core::timestamp::Timestamp;
use ibc::core::{ContextError, ExecutionContext, ValidationContext};
use ibc::proto::Any;
use ibc::Height;
use ics11_sov_celestia::client_state::{AnyClientState, SOVEREIGN_CLIENT_STATE_TYPE_URL};
use ics11_sov_celestia::consensus_state::AnyConsensusState;
use ics11_sov_celestia::proto::{
    ClientState as RawSovClientState, ConsensusState as RawSovConsensusState,
};
use tendermint_proto::Protobuf;

use super::definition::ContextMut;
use super::{ContextRef, StorageRef};

impl ValidationContext for ContextMut<'_> {
    type V = Self;
    type E = Self;
    type AnyConsensusState = AnyConsensusState;
    type AnyClientState = AnyClientState;

    fn get_client_validation_context(&self) -> &Self::V {
        self
    }

    fn client_state(&self, client_id: &ClientId) -> Result<Self::AnyClientState, ContextError> {
        client_state(self, client_id)
    }

    fn decode_client_state(&self, client_state: Any) -> Result<Self::AnyClientState, ContextError> {
        decode_client_state(self, client_state)
    }

    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::AnyConsensusState, ContextError> {
        consensus_state(self, client_cons_state_path)
    }

    fn host_height(&self) -> Result<Height, ContextError> {
        let host_height = Height::new(0, self.env.block.height)?;

        Ok(host_height)
    }

    fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
        let time = self.env.block.time;
        let host_timestamp = Timestamp::from_nanoseconds(time.nanos()).expect("invalid timestamp");

        Ok(host_timestamp)
    }

    fn host_consensus_state(
        &self,
        _height: &Height,
    ) -> Result<Self::AnyConsensusState, ContextError> {
        unimplemented!()
    }

    fn client_counter(&self) -> Result<u64, ContextError> {
        unimplemented!()
    }

    fn connection_end(&self, _conn_id: &ConnectionId) -> Result<ConnectionEnd, ContextError> {
        unimplemented!()
    }

    fn validate_self_client(
        &self,
        _client_state_of_host_on_counterparty: Any,
    ) -> Result<(), ContextError> {
        Ok(())
    }

    fn commitment_prefix(&self) -> CommitmentPrefix {
        unimplemented!()
    }

    fn connection_counter(&self) -> Result<u64, ContextError> {
        unimplemented!()
    }

    fn channel_end(&self, _channel_end_path: &ChannelEndPath) -> Result<ChannelEnd, ContextError> {
        unimplemented!()
    }

    fn get_next_sequence_send(
        &self,
        _seq_send_path: &SeqSendPath,
    ) -> Result<Sequence, ContextError> {
        unimplemented!()
    }

    fn get_next_sequence_recv(
        &self,
        _seq_recv_path: &SeqRecvPath,
    ) -> Result<Sequence, ContextError> {
        unimplemented!()
    }

    fn get_next_sequence_ack(&self, _seq_ack_path: &SeqAckPath) -> Result<Sequence, ContextError> {
        unimplemented!()
    }

    fn get_packet_commitment(
        &self,
        _commitment_path: &CommitmentPath,
    ) -> Result<PacketCommitment, ContextError> {
        unimplemented!()
    }

    fn get_packet_receipt(&self, _receipt_path: &ReceiptPath) -> Result<Receipt, ContextError> {
        unimplemented!()
    }

    fn get_packet_acknowledgement(
        &self,
        _ack_path: &AckPath,
    ) -> Result<AcknowledgementCommitment, ContextError> {
        unimplemented!()
    }

    fn channel_counter(&self) -> Result<u64, ContextError> {
        unimplemented!()
    }

    fn max_expected_time_per_block(&self) -> Duration {
        // This effectively cancels the check on connection block delays.
        Duration::ZERO
    }

    fn validate_message_signer(&self, _signer: &ibc::Signer) -> Result<(), ContextError> {
        Ok(())
    }
}

impl ExecutionContext for ContextMut<'_> {
    fn get_client_execution_context(&mut self) -> &mut Self::E {
        self
    }

    fn increase_client_counter(&mut self) -> Result<(), ContextError> {
        unimplemented!()
    }

    fn store_connection(
        &mut self,
        _connection_path: &ConnectionPath,
        _connection_end: ConnectionEnd,
    ) -> Result<(), ContextError> {
        unimplemented!()
    }

    fn store_connection_to_client(
        &mut self,
        _client_connection_path: &ClientConnectionPath,
        _conn_id: ConnectionId,
    ) -> Result<(), ContextError> {
        unimplemented!()
    }

    fn increase_connection_counter(&mut self) -> Result<(), ContextError> {
        unimplemented!()
    }

    fn store_packet_commitment(
        &mut self,
        _commitment_path: &CommitmentPath,
        _commitment: PacketCommitment,
    ) -> Result<(), ContextError> {
        unimplemented!()
    }

    fn delete_packet_commitment(
        &mut self,
        _commitment_path: &CommitmentPath,
    ) -> Result<(), ContextError> {
        unimplemented!()
    }

    fn store_packet_receipt(
        &mut self,
        _receipt_path: &ReceiptPath,
        _receipt: Receipt,
    ) -> Result<(), ContextError> {
        unimplemented!()
    }

    fn store_packet_acknowledgement(
        &mut self,
        _ack_path: &AckPath,
        _ack_commitment: AcknowledgementCommitment,
    ) -> Result<(), ContextError> {
        unimplemented!()
    }

    fn delete_packet_acknowledgement(&mut self, _ack_path: &AckPath) -> Result<(), ContextError> {
        unimplemented!()
    }

    fn store_channel(
        &mut self,
        _channel_end_path: &ChannelEndPath,
        _channel_end: ChannelEnd,
    ) -> Result<(), ContextError> {
        unimplemented!()
    }

    fn store_next_sequence_send(
        &mut self,
        _seq_send_path: &SeqSendPath,
        _seq: Sequence,
    ) -> Result<(), ContextError> {
        unimplemented!()
    }

    fn store_next_sequence_recv(
        &mut self,
        _seq_recv_path: &SeqRecvPath,
        _seq: Sequence,
    ) -> Result<(), ContextError> {
        unimplemented!()
    }

    fn store_next_sequence_ack(
        &mut self,
        _seq_ack_path: &SeqAckPath,
        _seq: Sequence,
    ) -> Result<(), ContextError> {
        unimplemented!()
    }

    fn increase_channel_counter(&mut self) -> Result<(), ContextError> {
        unimplemented!()
    }

    fn emit_ibc_event(&mut self, _event: IbcEvent) -> Result<(), ContextError> {
        unimplemented!()
    }

    fn log_message(&mut self, _message: String) -> Result<(), ContextError> {
        unimplemented!()
    }
}

impl ValidationContext for ContextRef<'_> {
    type V = Self;
    type E = Self;
    type AnyConsensusState = AnyConsensusState;
    type AnyClientState = AnyClientState;

    fn get_client_validation_context(&self) -> &Self::V {
        self
    }

    fn client_state(&self, client_id: &ClientId) -> Result<Self::AnyClientState, ContextError> {
        client_state(self, client_id)
    }

    fn decode_client_state(&self, client_state: Any) -> Result<Self::AnyClientState, ContextError> {
        decode_client_state(self, client_state)
    }

    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::AnyConsensusState, ContextError> {
        consensus_state(self, client_cons_state_path)
    }

    fn host_height(&self) -> Result<Height, ContextError> {
        let host_height = Height::new(0, self.env.block.height)?;

        Ok(host_height)
    }

    fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
        let time = self.env.block.time;
        let host_timestamp = Timestamp::from_nanoseconds(time.nanos()).expect("invalid timestamp");

        Ok(host_timestamp)
    }

    fn host_consensus_state(
        &self,
        _height: &Height,
    ) -> Result<Self::AnyConsensusState, ContextError> {
        unimplemented!()
    }

    fn client_counter(&self) -> Result<u64, ContextError> {
        unimplemented!()
    }

    fn connection_end(&self, _conn_id: &ConnectionId) -> Result<ConnectionEnd, ContextError> {
        unimplemented!()
    }

    fn validate_self_client(
        &self,
        _client_state_of_host_on_counterparty: Any,
    ) -> Result<(), ContextError> {
        Ok(())
    }

    fn commitment_prefix(&self) -> CommitmentPrefix {
        unimplemented!()
    }

    fn connection_counter(&self) -> Result<u64, ContextError> {
        unimplemented!()
    }

    fn channel_end(&self, _channel_end_path: &ChannelEndPath) -> Result<ChannelEnd, ContextError> {
        unimplemented!()
    }

    fn get_next_sequence_send(
        &self,
        _seq_send_path: &SeqSendPath,
    ) -> Result<Sequence, ContextError> {
        unimplemented!()
    }

    fn get_next_sequence_recv(
        &self,
        _seq_recv_path: &SeqRecvPath,
    ) -> Result<Sequence, ContextError> {
        unimplemented!()
    }

    fn get_next_sequence_ack(&self, _seq_ack_path: &SeqAckPath) -> Result<Sequence, ContextError> {
        unimplemented!()
    }

    fn get_packet_commitment(
        &self,
        _commitment_path: &CommitmentPath,
    ) -> Result<PacketCommitment, ContextError> {
        unimplemented!()
    }

    fn get_packet_receipt(&self, _receipt_path: &ReceiptPath) -> Result<Receipt, ContextError> {
        unimplemented!()
    }

    fn get_packet_acknowledgement(
        &self,
        _ack_path: &AckPath,
    ) -> Result<AcknowledgementCommitment, ContextError> {
        unimplemented!()
    }

    fn channel_counter(&self) -> Result<u64, ContextError> {
        unimplemented!()
    }

    fn max_expected_time_per_block(&self) -> Duration {
        // This effectively cancels the check on connection block delays.
        Duration::ZERO
    }

    fn validate_message_signer(&self, _signer: &ibc::Signer) -> Result<(), ContextError> {
        Ok(())
    }
}

pub fn client_state<Ctx>(ctx: &Ctx, client_id: &ClientId) -> Result<AnyClientState, ContextError>
where
    Ctx: ValidationContext + StorageRef,
{
    let client_state_path = ClientStatePath::new(client_id).to_string();

    let client_state_value =
        ctx.storage()
            .get(client_state_path.as_bytes())
            .ok_or(ClientError::Other {
                description: "Client state not found".to_string(),
            })?;

    let sov_client_state = Protobuf::<RawSovClientState>::decode(client_state_value.as_slice())
        .map_err(|e| ClientError::Other {
            description: e.to_string(),
        })?;

    Ok(AnyClientState::Sovereign(sov_client_state))
}

fn decode_client_state<Ctx>(_ctx: &Ctx, client_state: Any) -> Result<AnyClientState, ContextError>
where
    Ctx: ValidationContext + StorageRef,
{
    match client_state.type_url.as_str() {
        SOVEREIGN_CLIENT_STATE_TYPE_URL => {
            let sov_client_state = Protobuf::<RawSovClientState>::decode(
                client_state.value.as_slice(),
            )
            .map_err(|e| ClientError::Other {
                description: e.to_string(),
            })?;

            Ok(AnyClientState::Sovereign(sov_client_state))
        }
        _ => Err(ClientError::Other {
            description: "Client state type not supported".to_string(),
        }
        .into()),
    }
}

fn consensus_state<Ctx>(
    ctx: &Ctx,
    client_cons_state_path: &ClientConsensusStatePath,
) -> Result<AnyConsensusState, ContextError>
where
    Ctx: ValidationContext + StorageRef,
{
    let consensus_state_value = ctx
        .storage()
        .get(client_cons_state_path.to_string().as_bytes())
        .ok_or(ClientError::Other {
            description: "Consensus state not found".to_string(),
        })?;

    let consensus_state = Protobuf::<RawSovConsensusState>::decode(
        consensus_state_value.as_slice(),
    )
    .map_err(|e| ClientError::Other {
        description: e.to_string(),
    })?;

    Ok(AnyConsensusState::Sovereign(consensus_state))
}
