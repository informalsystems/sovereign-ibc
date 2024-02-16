use std::time::Duration;

use ibc_core::channel::types::channel::ChannelEnd;
use ibc_core::channel::types::commitment::{AcknowledgementCommitment, PacketCommitment};
use ibc_core::channel::types::packet::Receipt;
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Height;
use ibc_core::commitment_types::commitment::CommitmentPrefix;
use ibc_core::connection::types::ConnectionEnd;
use ibc_core::handler::types::error::ContextError;
use ibc_core::handler::types::events::IbcEvent;
use ibc_core::host::types::identifiers::{ClientId, ConnectionId, Sequence};
use ibc_core::host::types::path::{
    AckPath, ChannelEndPath, ClientConnectionPath, ClientConsensusStatePath, ClientStatePath,
    CommitmentPath, ConnectionPath, ReceiptPath, SeqAckPath, SeqRecvPath, SeqSendPath,
};
use ibc_core::host::{ExecutionContext, ValidationContext};
use ibc_core::primitives::proto::{Any, Protobuf};
use ibc_core::primitives::{Signer, Timestamp};
use sov_celestia_client::client_state::ClientState;
use sov_celestia_client::types::client_state::SOV_TENDERMINT_CLIENT_STATE_TYPE_URL;
use sov_celestia_client::types::proto::v1::{
    ClientState as RawClientState, ConsensusState as RawConsensusState,
};

use super::Context;
use crate::types::AnyConsensusState;

impl ValidationContext for Context<'_> {
    type V = Self;
    type E = Self;
    type AnyConsensusState = AnyConsensusState;
    type AnyClientState = ClientState;

    fn get_client_validation_context(&self) -> &Self::V {
        self
    }

    fn client_state(&self, _client_id: &ClientId) -> Result<Self::AnyClientState, ContextError> {
        let client_state_value = self.retrieve(ClientStatePath::leaf())?;

        let sov_client_state = Protobuf::<RawClientState>::decode(client_state_value.as_slice())
            .map_err(|e| ClientError::Other {
                description: e.to_string(),
            })?;

        Ok(sov_client_state)
    }

    fn decode_client_state(&self, client_state: Any) -> Result<Self::AnyClientState, ContextError> {
        match client_state.type_url.as_str() {
            SOV_TENDERMINT_CLIENT_STATE_TYPE_URL => {
                let sov_client_state = Protobuf::<RawClientState>::decode(
                    client_state.value.as_slice(),
                )
                .map_err(|e| ClientError::Other {
                    description: e.to_string(),
                })?;

                Ok(sov_client_state)
            }
            _ => Err(ClientError::Other {
                description: "Client state type not supported".to_string(),
            }
            .into()),
        }
    }

    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::AnyConsensusState, ContextError> {
        let consensus_state_value = self.retrieve(client_cons_state_path.leaf())?;

        let consensus_state = Protobuf::<RawConsensusState>::decode(
            consensus_state_value.as_slice(),
        )
        .map_err(|e| ClientError::Other {
            description: e.to_string(),
        })?;

        Ok(AnyConsensusState::Sovereign(consensus_state))
    }

    fn host_height(&self) -> Result<Height, ContextError> {
        let host_height = Height::new(0, self.env().block.height)?;

        Ok(host_height)
    }

    fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
        let time = self.env().block.time;

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

    fn validate_message_signer(&self, _signer: &Signer) -> Result<(), ContextError> {
        Ok(())
    }
}

impl ExecutionContext for Context<'_> {
    fn get_client_execution_context(&mut self) -> &mut Self::E {
        todo!()
    }

    fn increase_client_counter(&mut self) -> Result<(), ContextError> {
        todo!()
    }

    fn store_connection(
        &mut self,
        _connection_path: &ConnectionPath,
        _connection_end: ConnectionEnd,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_connection_to_client(
        &mut self,
        _client_connection_path: &ClientConnectionPath,
        _conn_id: ConnectionId,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn increase_connection_counter(&mut self) -> Result<(), ContextError> {
        todo!()
    }

    fn store_packet_commitment(
        &mut self,
        _commitment_path: &CommitmentPath,
        _commitment: PacketCommitment,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn delete_packet_commitment(
        &mut self,
        _commitment_path: &CommitmentPath,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_packet_receipt(
        &mut self,
        _receipt_path: &ReceiptPath,
        _receipt: Receipt,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_packet_acknowledgement(
        &mut self,
        _ack_path: &AckPath,
        _ack_commitment: AcknowledgementCommitment,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn delete_packet_acknowledgement(&mut self, _ack_path: &AckPath) -> Result<(), ContextError> {
        todo!()
    }

    fn store_channel(
        &mut self,
        _channel_end_path: &ChannelEndPath,
        _channel_end: ChannelEnd,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_next_sequence_send(
        &mut self,
        _seq_send_path: &SeqSendPath,
        _seq: Sequence,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_next_sequence_recv(
        &mut self,
        _seq_recv_path: &SeqRecvPath,
        _seq: Sequence,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_next_sequence_ack(
        &mut self,
        _seq_ack_path: &SeqAckPath,
        _seq: Sequence,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn increase_channel_counter(&mut self) -> Result<(), ContextError> {
        todo!()
    }

    fn emit_ibc_event(&mut self, _event: IbcEvent) -> Result<(), ContextError> {
        todo!()
    }

    fn log_message(&mut self, _message: String) -> Result<(), ContextError> {
        todo!()
    }
}
