pub mod clients;

use core::time::Duration;
use std::cell::RefCell;
use std::rc::Rc;

use ibc::clients::ics07_tendermint::client_state::ClientState as TmClientState;
use ibc::core::events::IbcEvent;
use ibc::core::ics02_client::error::ClientError;
use ibc::core::ics03_connection::connection::ConnectionEnd;
use ibc::core::ics03_connection::error::ConnectionError;
use ibc::core::ics04_channel::channel::ChannelEnd;
use ibc::core::ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment};
use ibc::core::ics04_channel::error::{ChannelError, PacketError};
use ibc::core::ics04_channel::packet::{Receipt, Sequence};
use ibc::core::ics23_commitment::commitment::CommitmentPrefix;
use ibc::core::ics24_host::identifier::{ClientId, ConnectionId};
use ibc::core::ics24_host::path::{
    AckPath, ChannelEndPath, ClientConnectionPath, ClientConsensusStatePath, CommitmentPath,
    ConnectionPath, ReceiptPath, SeqAckPath, SeqRecvPath, SeqSendPath,
};
use ibc::core::timestamp::Timestamp;
use ibc::core::{ContextError, ExecutionContext, ValidationContext};
use ibc::Height;
use sov_state::WorkingSet;

use crate::Ibc;

/// The SDK doesn't have a concept of a "revision number", so we default to 1
const HOST_REVISION_NUMBER: u64 = 1;

pub struct IbcExecutionContext<'a, C, Da>
where
    C: sov_modules_api::Context,
    Da: sov_modules_api::DaSpec,
{
    pub ibc: &'a Ibc<C, Da>,
    pub working_set: Rc<RefCell<&'a mut WorkingSet<C::Storage>>>,
}

impl<'a, C, Da> ValidationContext for IbcExecutionContext<'a, C, Da>
where
    C: sov_modules_api::Context,
    Da: sov_modules_api::DaSpec,
{
    type ClientValidationContext = Self;
    type E = Self;
    type AnyConsensusState = clients::AnyConsensusState;
    type AnyClientState = clients::AnyClientState;

    fn get_client_validation_context(&self) -> &Self::ClientValidationContext {
        self
    }

    fn client_state(&self, client_id: &ClientId) -> Result<Self::AnyClientState, ContextError> {
        self.ibc
            .client_state_map
            .get(client_id, *self.working_set.borrow_mut())
            .ok_or(
                ClientError::ClientStateNotFound {
                    client_id: client_id.clone(),
                }
                .into(),
            )
    }

    fn decode_client_state(
        &self,
        client_state: ibc::Any,
    ) -> Result<Self::AnyClientState, ContextError> {
        let tm_client_state: TmClientState = client_state.try_into()?;

        Ok(tm_client_state.into())
    }

    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::AnyConsensusState, ContextError> {
        self.ibc
            .consensus_state_map
            .get(client_cons_state_path, *self.working_set.borrow_mut())
            .ok_or(
                ClientError::ConsensusStateNotFound {
                    client_id: client_cons_state_path.client_id.clone(),
                    height: Height::new(
                        client_cons_state_path.epoch,
                        client_cons_state_path.height,
                    )
                    .map_err(|_| ClientError::InvalidHeight)?,
                }
                .into(),
            )
    }

    fn client_update_time(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Timestamp, ContextError> {
        self.ibc
            .client_update_times_map
            .get(
                &(client_id.clone(), *height),
                *self.working_set.borrow_mut(),
            )
            .ok_or(
                ClientError::Other {
                    description: "Client update time not found".to_string(),
                }
                .into(),
            )
    }

    fn client_update_height(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Height, ContextError> {
        self.ibc
            .client_update_heights_map
            .get(
                &(client_id.clone(), *height),
                *self.working_set.borrow_mut(),
            )
            .ok_or(
                ClientError::Other {
                    description: "Client update time not found".to_string(),
                }
                .into(),
            )
    }

    fn host_height(&self) -> Result<Height, ContextError> {
        let slot_height = self
            .ibc
            .chain_state
            .get_slot_height(&mut self.working_set.borrow_mut());

        Ok(Height::new(HOST_REVISION_NUMBER, slot_height)?)
    }

    fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
        let chain_time = self
            .ibc
            .chain_state
            .get_time(&mut self.working_set.borrow_mut());

        if chain_time.secs() < 0 {
            // FIXME: at least add a `ContextError::Host` enum variant, and use that here
            return Err(ContextError::ClientError(ClientError::Other {
                description: format!("Invalid host chain time: {}", chain_time.secs()),
            }));
        }

        let time_in_nanos: u64 =
            (chain_time.secs() as u64) * 10u64.pow(9) + chain_time.subsec_nanos() as u64;

        // FIXME: at least add a `ContextError::Host` enum variant, and use that here
        let timestamp = Timestamp::from_nanoseconds(time_in_nanos)
            .map_err(PacketError::InvalidPacketTimestamp)?;

        Ok(timestamp)
    }

    fn host_consensus_state(
        &self,
        height: &Height,
    ) -> Result<Self::AnyConsensusState, ContextError> {
        // TODO: In order to implement this, we need to first define the
        // `ConsensusState` protobuf definition that SDK chains will use
        todo!()
    }

    fn client_counter(&self) -> Result<u64, ContextError> {
        self.ibc
            .client_counter
            .get(*self.working_set.borrow_mut())
            .ok_or(
                ClientError::Other {
                    description: "Client counter not found".to_string(),
                }
                .into(),
            )
    }

    fn connection_end(&self, conn_id: &ConnectionId) -> Result<ConnectionEnd, ContextError> {
        self.ibc
            .connection_end_map
            .get(
                &ConnectionPath::new(conn_id),
                *self.working_set.borrow_mut(),
            )
            .ok_or(
                ConnectionError::ConnectionNotFound {
                    connection_id: conn_id.clone(),
                }
                .into(),
            )
    }

    fn validate_self_client(
        &self,
        client_state_of_host_on_counterparty: ibc::Any,
    ) -> Result<(), ContextError> {
        // Note: We can optionally implement this.
        // It would require having a Protobuf definition of the chain's `ClientState` that other chains would use.
        // The relayer sends us this `ClientState` as stored on other chains, and we validate it here.
        Ok(())
    }

    // As modules presently lack direct access to their own prefixes, we
    // truncate the prefix of a field (e.g. client_counter) in order to derive
    // the module's prefix.
    fn commitment_prefix(&self) -> CommitmentPrefix {
        let client_counter_prefix = self.ibc.client_counter.prefix();

        let client_counter_prefix_vec = client_counter_prefix.as_aligned_vec().as_ref();

        let module_prefix_len = client_counter_prefix.len() - b"client_counter/".len();

        let module_prefix = client_counter_prefix_vec[..module_prefix_len].to_vec();

        CommitmentPrefix::try_from(module_prefix).expect("never fails as prefix is not empty")
    }

    fn connection_counter(&self) -> Result<u64, ContextError> {
        self.ibc
            .connection_counter
            .get(*self.working_set.borrow_mut())
            .ok_or(
                ConnectionError::Other {
                    description: "Connection counter not found".to_string(),
                }
                .into(),
            )
    }

    fn channel_end(&self, channel_end_path: &ChannelEndPath) -> Result<ChannelEnd, ContextError> {
        self.ibc
            .channel_end_map
            .get(channel_end_path, *self.working_set.borrow_mut())
            .ok_or(
                ChannelError::ChannelNotFound {
                    port_id: channel_end_path.0.clone(),
                    channel_id: channel_end_path.1.clone(),
                }
                .into(),
            )
    }

    fn get_next_sequence_send(
        &self,
        seq_send_path: &SeqSendPath,
    ) -> Result<Sequence, ContextError> {
        self.ibc
            .send_sequence_map
            .get(seq_send_path, *self.working_set.borrow_mut())
            .ok_or(
                PacketError::MissingNextSendSeq {
                    port_id: seq_send_path.0.clone(),
                    channel_id: seq_send_path.1.clone(),
                }
                .into(),
            )
    }

    fn get_next_sequence_recv(
        &self,
        seq_recv_path: &SeqRecvPath,
    ) -> Result<Sequence, ContextError> {
        self.ibc
            .recv_sequence_map
            .get(seq_recv_path, *self.working_set.borrow_mut())
            .ok_or(
                PacketError::MissingNextRecvSeq {
                    port_id: seq_recv_path.0.clone(),
                    channel_id: seq_recv_path.1.clone(),
                }
                .into(),
            )
    }

    fn get_next_sequence_ack(&self, seq_ack_path: &SeqAckPath) -> Result<Sequence, ContextError> {
        self.ibc
            .ack_sequence_map
            .get(seq_ack_path, *self.working_set.borrow_mut())
            .ok_or(
                PacketError::MissingNextAckSeq {
                    port_id: seq_ack_path.0.clone(),
                    channel_id: seq_ack_path.1.clone(),
                }
                .into(),
            )
    }

    fn get_packet_commitment(
        &self,
        commitment_path: &CommitmentPath,
    ) -> Result<PacketCommitment, ContextError> {
        self.ibc
            .packet_commitment_map
            .get(commitment_path, *self.working_set.borrow_mut())
            .ok_or(
                PacketError::PacketCommitmentNotFound {
                    sequence: commitment_path.sequence,
                }
                .into(),
            )
    }

    fn get_packet_receipt(&self, receipt_path: &ReceiptPath) -> Result<Receipt, ContextError> {
        self.ibc
            .packet_receipt_map
            .get(receipt_path, *self.working_set.borrow_mut())
            .ok_or(
                PacketError::PacketReceiptNotFound {
                    sequence: receipt_path.sequence,
                }
                .into(),
            )
    }

    fn get_packet_acknowledgement(
        &self,
        ack_path: &AckPath,
    ) -> Result<AcknowledgementCommitment, ContextError> {
        self.ibc
            .packet_ack_map
            .get(ack_path, *self.working_set.borrow_mut())
            .ok_or(
                PacketError::PacketAcknowledgementNotFound {
                    sequence: ack_path.sequence,
                }
                .into(),
            )
    }

    fn channel_counter(&self) -> Result<u64, ContextError> {
        self.ibc
            .channel_counter
            .get(*self.working_set.borrow_mut())
            .ok_or(
                ChannelError::Other {
                    description: "Channel counter not found".to_string(),
                }
                .into(),
            )
    }

    fn max_expected_time_per_block(&self) -> Duration {
        // This effectively cancels the check on connection block delays. Not
        // all DAs have predictable block times (such as Bitcoin and Avalanche),
        // so we cannot support connection block delays as they are defined
        // today.
        Duration::ZERO
    }

    fn validate_message_signer(&self, signer: &ibc::Signer) -> Result<(), ContextError> {
        Ok(())
    }
}

impl<'a, C, Da> ExecutionContext for IbcExecutionContext<'a, C, Da>
where
    C: sov_modules_api::Context,
    Da: sov_modules_api::DaSpec,
{
    fn get_client_execution_context(&mut self) -> &mut Self::E {
        self
    }

    fn increase_client_counter(&mut self) -> Result<(), ContextError> {
        let next_client_counter = self
            .ibc
            .client_counter
            .get(*self.working_set.borrow_mut())
            .ok_or(ClientError::Other {
                description: "Client counter not found".to_string(),
            })?
            .checked_add(1)
            .ok_or(ClientError::CounterOverflow)?;

        self.ibc
            .client_counter
            .set(&next_client_counter, *self.working_set.borrow_mut());

        Ok(())
    }

    fn store_update_time(
        &mut self,
        client_id: ClientId,
        height: Height,
        timestamp: Timestamp,
    ) -> Result<(), ContextError> {
        self.ibc.client_update_times_map.set(
            &(client_id, height),
            &timestamp,
            *self.working_set.borrow_mut(),
        );
        Ok(())
    }

    fn store_update_height(
        &mut self,
        client_id: ClientId,
        height: Height,
        host_height: Height,
    ) -> Result<(), ContextError> {
        self.ibc.client_update_heights_map.set(
            &(client_id, height),
            &host_height,
            *self.working_set.borrow_mut(),
        );
        Ok(())
    }

    fn store_connection(
        &mut self,
        connection_path: &ConnectionPath,
        connection_end: ConnectionEnd,
    ) -> Result<(), ContextError> {
        self.ibc.connection_end_map.set(
            connection_path,
            &connection_end,
            *self.working_set.borrow_mut(),
        );
        Ok(())
    }

    fn store_connection_to_client(
        &mut self,
        client_connection_path: &ClientConnectionPath,
        conn_id: ConnectionId,
    ) -> Result<(), ContextError> {
        let mut connection_ids = self
            .ibc
            .connection_ids_map
            .get(client_connection_path, *self.working_set.borrow_mut())
            .unwrap_or_default();

        connection_ids.push(conn_id);

        self.ibc.connection_ids_map.set(
            client_connection_path,
            &connection_ids,
            *self.working_set.borrow_mut(),
        );

        Ok(())
    }

    fn increase_connection_counter(&mut self) -> Result<(), ContextError> {
        let next_connection_counter = self
            .ibc
            .connection_counter
            .get(*self.working_set.borrow_mut())
            .ok_or(ConnectionError::Other {
                description: "Connection counter not found".to_string(),
            })?
            .checked_add(1)
            .ok_or(ConnectionError::CounterOverflow)?;

        self.ibc
            .connection_counter
            .set(&next_connection_counter, *self.working_set.borrow_mut());

        Ok(())
    }

    fn store_packet_commitment(
        &mut self,
        commitment_path: &CommitmentPath,
        commitment: PacketCommitment,
    ) -> Result<(), ContextError> {
        self.ibc.packet_commitment_map.set(
            commitment_path,
            &commitment,
            *self.working_set.borrow_mut(),
        );
        Ok(())
    }

    fn delete_packet_commitment(
        &mut self,
        commitment_path: &CommitmentPath,
    ) -> Result<(), ContextError> {
        self.ibc
            .packet_commitment_map
            .delete(commitment_path, *self.working_set.borrow_mut());
        Ok(())
    }

    fn store_packet_receipt(
        &mut self,
        receipt_path: &ReceiptPath,
        receipt: Receipt,
    ) -> Result<(), ContextError> {
        self.ibc
            .packet_receipt_map
            .set(receipt_path, &receipt, *self.working_set.borrow_mut());
        Ok(())
    }

    fn store_packet_acknowledgement(
        &mut self,
        ack_path: &AckPath,
        ack_commitment: AcknowledgementCommitment,
    ) -> Result<(), ContextError> {
        self.ibc
            .packet_ack_map
            .set(ack_path, &ack_commitment, *self.working_set.borrow_mut());
        Ok(())
    }

    fn delete_packet_acknowledgement(&mut self, ack_path: &AckPath) -> Result<(), ContextError> {
        self.ibc
            .packet_ack_map
            .delete(ack_path, *self.working_set.borrow_mut());
        Ok(())
    }

    fn store_channel(
        &mut self,
        channel_end_path: &ChannelEndPath,
        channel_end: ChannelEnd,
    ) -> Result<(), ContextError> {
        self.ibc.channel_end_map.set(
            channel_end_path,
            &channel_end,
            *self.working_set.borrow_mut(),
        );
        Ok(())
    }

    fn store_next_sequence_send(
        &mut self,
        seq_send_path: &SeqSendPath,
        seq: Sequence,
    ) -> Result<(), ContextError> {
        self.ibc
            .send_sequence_map
            .set(seq_send_path, &seq, *self.working_set.borrow_mut());
        Ok(())
    }

    fn store_next_sequence_recv(
        &mut self,
        seq_recv_path: &SeqRecvPath,
        seq: Sequence,
    ) -> Result<(), ContextError> {
        self.ibc
            .recv_sequence_map
            .set(seq_recv_path, &seq, *self.working_set.borrow_mut());
        Ok(())
    }

    fn store_next_sequence_ack(
        &mut self,
        seq_ack_path: &SeqAckPath,
        seq: Sequence,
    ) -> Result<(), ContextError> {
        self.ibc
            .ack_sequence_map
            .set(seq_ack_path, &seq, *self.working_set.borrow_mut());
        Ok(())
    }

    fn increase_channel_counter(&mut self) -> Result<(), ContextError> {
        let next_channel_counter = self
            .ibc
            .channel_counter
            .get(*self.working_set.borrow_mut())
            .ok_or(ChannelError::Other {
                description: "Channel counter not found".to_string(),
            })?
            .checked_add(1)
            .ok_or(ChannelError::CounterOverflow)?;

        self.ibc
            .channel_counter
            .set(&next_channel_counter, *self.working_set.borrow_mut());

        Ok(())
    }

    fn emit_ibc_event(&mut self, event: IbcEvent) -> Result<(), ContextError> {
        // Note: as an interim solution, we transform IBC events into Tendermint
        // events to simplify the conversion process, avoiding the need for
        // converting individual IBC event types into a key-value pair of `&str`
        let tm_event =
            tendermint::abci::Event::try_from(event).map_err(|_| ClientError::Other {
                description: "Failed to convert IBC event to Tendermint event".to_string(),
            })?;

        let event_attribute: Vec<String> = tm_event
            .attributes
            .into_iter()
            .map(|attr| format!("{attr:?}"))
            .collect();

        self.working_set
            .borrow_mut()
            .add_event(tm_event.kind.as_str(), event_attribute.join(",").as_str());

        Ok(())
    }

    /// FIXME: To implement this method there should be a way for IBC module to
    /// insert logs into the transaction receipts upon execution
    fn log_message(&mut self, message: String) -> Result<(), ContextError> {
        Ok(())
    }
}
