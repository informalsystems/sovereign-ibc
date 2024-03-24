use core::time::Duration;
use std::cell::RefCell;
use std::rc::Rc;

use ibc_core::channel::types::channel::ChannelEnd;
use ibc_core::channel::types::commitment::{AcknowledgementCommitment, PacketCommitment};
use ibc_core::channel::types::error::{ChannelError, PacketError};
use ibc_core::channel::types::packet::Receipt;
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Height;
use ibc_core::commitment_types::commitment::CommitmentPrefix;
use ibc_core::connection::types::error::ConnectionError;
use ibc_core::connection::types::ConnectionEnd;
use ibc_core::handler::types::error::ContextError;
use ibc_core::handler::types::events::IbcEvent;
use ibc_core::host::types::identifiers::{ConnectionId, Sequence};
use ibc_core::host::types::path::{
    AckPath, ChannelEndPath, ClientConnectionPath, CommitmentPath, ConnectionPath, ReceiptPath,
    SeqAckPath, SeqRecvPath, SeqSendPath,
};
use ibc_core::host::{ExecutionContext, ValidationContext};
use ibc_core::primitives::{Signer, Timestamp};
use sov_celestia_client::client_state::ClientState;
use sov_celestia_client::consensus_state::ConsensusState;
use sov_modules_api::{Context, DaSpec, EventEmitter, ModuleInfo, Spec, WorkingSet};
use sov_state::Prefix;

use crate::event::auxiliary_packet_events;
use crate::Ibc;

/// The SDK doesn't have a concept of a "revision number", so we default to 0
pub const HOST_REVISION_NUMBER: u64 = 0;

#[derive(Clone)]
pub struct IbcContext<'a, S, Da>
where
    S: Spec,
    Da: DaSpec,
{
    pub ibc: &'a Ibc<S, Da>,
    pub context: Option<Context<S>>,
    pub working_set: Rc<RefCell<&'a mut WorkingSet<S>>>,
}

impl<'a, S, Da> IbcContext<'a, S, Da>
where
    S: Spec,
    Da: DaSpec,
{
    pub fn new(
        ibc: &'a Ibc<S, Da>,
        context: Option<Context<S>>,
        working_set: Rc<RefCell<&'a mut WorkingSet<S>>>,
    ) -> IbcContext<'a, S, Da> {
        IbcContext {
            ibc,
            context,
            working_set,
        }
    }
}

impl<'a, S, Da> ValidationContext for IbcContext<'a, S, Da>
where
    S: Spec,
    Da: DaSpec,
{
    type V = Self;
    type HostClientState = ClientState;
    type HostConsensusState = ConsensusState;

    fn get_client_validation_context(&self) -> &Self::V {
        self
    }

    fn host_height(&self) -> Result<Height, ContextError> {
        let context = self.context.clone().ok_or(ClientError::Other {
            description: "Context not found".to_string(),
        })?;

        Ok(Height::new(
            HOST_REVISION_NUMBER,
            context.visible_slot_number(),
        )?)
    }

    fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
        let host_consensus_state = self.host_consensus_state(&self.host_height()?)?;

        let chain_time = host_consensus_state.timestamp();

        if chain_time.unix_timestamp() < 0 {
            // FIXME: at least add a `ContextError::Host` enum variant, and use that here
            return Err(ContextError::ClientError(ClientError::Other {
                description: format!("Invalid host chain time: {}", chain_time.unix_timestamp()),
            }));
        }

        Ok(Timestamp::from(chain_time))
    }

    fn host_consensus_state(
        &self,
        height: &Height,
    ) -> Result<Self::HostConsensusState, ContextError> {
        let host_consensus_state = self
            .ibc
            .host_consensus_state_map
            .get(height, *self.working_set.borrow_mut())
            .ok_or(ClientError::Other {
                description: "Host consensus state not found".to_string(),
            })?;

        Ok(host_consensus_state)
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
        client_state_of_host_on_counterparty: Self::HostClientState,
    ) -> Result<(), ContextError> {
        // Note: We can optionally implement this.
        // It would require having a Protobuf definition of the chain's `ClientState` that other chains would use.
        // The relayer sends us this `ClientState` as stored on other chains, and we validate it here.
        Ok(())
    }

    fn commitment_prefix(&self) -> CommitmentPrefix {
        let module_prefix: Prefix = self.ibc.prefix().into();

        let module_prefix_vec = module_prefix.as_aligned_vec().clone().into_inner();

        CommitmentPrefix::try_from(module_prefix_vec).expect("never fails as prefix is not empty")
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

    fn validate_message_signer(&self, signer: &Signer) -> Result<(), ContextError> {
        Ok(())
    }
}

impl<'a, S, Da> ExecutionContext for IbcContext<'a, S, Da>
where
    S: Spec,
    Da: DaSpec,
{
    type E = Self;

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
            .client_connections_map
            .get(client_connection_path, *self.working_set.borrow_mut())
            .unwrap_or_default();

        connection_ids.push(conn_id);

        self.ibc.client_connections_map.set(
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
        self.ibc
            .packet_commitment_vec
            .push(commitment_path, *self.working_set.borrow_mut());
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
        let unprocessed_packets = self
            .ibc
            .packet_commitment_vec
            .iter(*self.working_set.borrow_mut())
            .filter(|path| path != commitment_path)
            .collect::<Vec<CommitmentPath>>();

        self.ibc
            .packet_commitment_vec
            .set_all(unprocessed_packets, *self.working_set.borrow_mut());

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
            .packet_receipt_vec
            .push(receipt_path, *self.working_set.borrow_mut());
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
            .packet_ack_vec
            .push(ack_path, *self.working_set.borrow_mut());
        self.ibc
            .packet_ack_map
            .set(ack_path, &ack_commitment, *self.working_set.borrow_mut());
        Ok(())
    }

    fn delete_packet_acknowledgement(&mut self, ack_path: &AckPath) -> Result<(), ContextError> {
        let filtered_acks = self
            .ibc
            .packet_ack_vec
            .iter(*self.working_set.borrow_mut())
            .filter(|path| path != ack_path)
            .collect::<Vec<AckPath>>();

        self.ibc
            .packet_ack_vec
            .set_all(filtered_acks, *self.working_set.borrow_mut());

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
        self.ibc.emit_event(
            *self.working_set.borrow_mut(),
            event.event_type(),
            event.clone(),
        );

        let events = auxiliary_packet_events(event)?;

        if !events.is_empty() {
            events.into_iter().for_each(|(event_key, event)| {
                self.ibc
                    .emit_event(*self.working_set.borrow_mut(), &event_key, event);
            });
        }

        Ok(())
    }

    /// FIXME: To implement this method there should be a way for IBC module to
    /// insert logs into the transaction receipts upon execution
    fn log_message(&mut self, message: String) -> Result<(), ContextError> {
        Ok(())
    }
}
