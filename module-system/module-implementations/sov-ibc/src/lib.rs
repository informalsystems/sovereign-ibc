#![allow(unused_variables)]
#![allow(dead_code)]

pub mod call;
pub mod codec;
pub mod genesis;

#[cfg(test)]
pub mod test_utils;

#[cfg(test)]
pub mod tests;

pub(crate) mod context;
mod router;

use codec::{AcknowledgementCommitmentCodec, PacketCommitmentCodec, ProtobufCodec};
use context::clients::{AnyClientState, AnyConsensusState};
use ibc::core::ics03_connection::connection::ConnectionEnd;
use ibc::core::ics04_channel::channel::ChannelEnd;
use ibc::core::ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment};
use ibc::core::ics04_channel::packet::{Receipt, Sequence};
use ibc::core::ics24_host::identifier::{ClientId, ConnectionId};
use ibc::core::ics24_host::path::{
    AckPath, ChannelEndPath, ClientConnectionPath, ClientConsensusStatePath, CommitmentPath,
    ConnectionPath, ReceiptPath, SeqAckPath, SeqRecvPath, SeqSendPath,
};
use ibc::core::timestamp::Timestamp;
use ibc::{Any, Height};
use ibc_proto::ibc::core::channel::v1::Channel as RawChannelEnd;
use ibc_proto::ibc::core::connection::v1::ConnectionEnd as RawConnectionEnd;
use sov_modules_api::{Context, DaSpec, Error, StateMap, StateValue, WorkingSet};
use sov_modules_macros::ModuleInfo;

pub struct ExampleModuleConfig {}

/// the sov-ibc module that manages all IBC-related states
///
/// Note: this struct, named `Ibc`, as its name serves as the module name and is
/// utilized to form prefixes for `state` fields. This naming adheres to the
/// module naming convention used throughout the codebase, ensuring created
/// prefixes by modules are in harmony.
#[derive(ModuleInfo, Clone)]
pub struct Ibc<C: sov_modules_api::Context, Da: sov_modules_api::DaSpec> {
    #[address]
    pub address: C::Address,

    #[module]
    pub(crate) transfer: sov_ibc_transfer::Transfer<C>,

    #[module]
    pub(crate) chain_state: sov_chain_state::ChainState<C, Da>,

    #[state]
    client_counter: StateValue<u64>,

    #[state]
    connection_counter: StateValue<u64>,

    #[state]
    channel_counter: StateValue<u64>,

    #[state]
    client_update_times_map: StateMap<(ClientId, Height), Timestamp>,

    #[state]
    client_update_heights_map: StateMap<(ClientId, Height), Height>,

    #[state]
    client_state_map: StateMap<ClientId, AnyClientState, ProtobufCodec<Any>>,

    #[state]
    consensus_state_map: StateMap<ClientConsensusStatePath, AnyConsensusState, ProtobufCodec<Any>>,

    #[state]
    connection_end_map: StateMap<ConnectionPath, ConnectionEnd, ProtobufCodec<RawConnectionEnd>>,

    #[state]
    connection_ids_map: StateMap<ClientConnectionPath, Vec<ConnectionId>>,

    #[state]
    channel_end_map: StateMap<ChannelEndPath, ChannelEnd, ProtobufCodec<RawChannelEnd>>,

    #[state]
    send_sequence_map: StateMap<SeqSendPath, Sequence>,

    #[state]
    recv_sequence_map: StateMap<SeqRecvPath, Sequence>,

    #[state]
    ack_sequence_map: StateMap<SeqAckPath, Sequence>,

    #[state]
    packet_commitment_map: StateMap<CommitmentPath, PacketCommitment, PacketCommitmentCodec>,

    #[state]
    packet_receipt_map: StateMap<ReceiptPath, Receipt>,

    #[state]
    packet_ack_map: StateMap<AckPath, AcknowledgementCommitment, AcknowledgementCommitmentCodec>,
}

impl<C: Context, Da: DaSpec> sov_modules_api::Module for Ibc<C, Da> {
    type Context = C;

    type Config = ExampleModuleConfig;

    type CallMessage = call::CallMessage<C>;

    fn genesis(&self, config: &Self::Config, working_set: &mut WorkingSet<C>) -> Result<(), Error> {
        // The initialization logic
        Ok(self.init_module(config, working_set)?)
    }

    fn call(
        &self,
        msg: Self::CallMessage,
        context: &Self::Context,
        working_set: &mut WorkingSet<C>,
    ) -> Result<sov_modules_api::CallResponse, Error> {
        match msg {
            call::CallMessage::Core(msg_envelope) => {
                Ok(self.process_core_message(msg_envelope, context, working_set)?)
            }
            call::CallMessage::Transfer(sdk_token_transfer) => {
                Ok(self.transfer(sdk_token_transfer, context, working_set)?)
            }
        }
    }
}
