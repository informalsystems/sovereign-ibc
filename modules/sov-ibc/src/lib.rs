#![allow(unused_variables)]
#![allow(dead_code)]

pub mod call;
pub mod clients;
pub mod codec;
pub mod genesis;

#[cfg(feature = "native")]
mod query;
#[cfg(feature = "native")]
pub use query::*;

pub mod context;
mod router;

use clients::{AnyClientState, AnyConsensusState};
use codec::{AcknowledgementCommitmentCodec, PacketCommitmentCodec, ProtobufCodec};
use ibc_core::channel::types::channel::ChannelEnd;
use ibc_core::channel::types::commitment::{AcknowledgementCommitment, PacketCommitment};
use ibc_core::channel::types::packet::Receipt;
use ibc_core::channel::types::proto::v1::Channel as RawChannelEnd;
use ibc_core::client::types::Height;
use ibc_core::connection::types::proto::v1::ConnectionEnd as RawConnectionEnd;
use ibc_core::connection::types::ConnectionEnd;
use ibc_core::host::types::identifiers::{ClientId, ConnectionId, Sequence};
use ibc_core::host::types::path::{
    AckPath, ChannelEndPath, ClientConnectionPath, ClientConsensusStatePath, CommitmentPath,
    ConnectionPath, ReceiptPath, SeqAckPath, SeqRecvPath, SeqSendPath,
};
use ibc_core::primitives::proto::Any;
use ibc_core::primitives::Timestamp;
use serde::{Deserialize, Serialize};
use sov_ibc_transfer::IbcTransfer;
use sov_modules_api::{Context, DaSpec, Error, StateMap, StateValue, StateVec, WorkingSet};
use sov_modules_macros::ModuleInfo;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ExampleModuleConfig {}

/// the sov-ibc module that manages all IBC-related states
///
/// Note: this struct, named `Ibc`, as its name serves as the module name and is
/// utilized to form prefixes for `state` fields. This naming adheres to the
/// module naming convention used throughout the codebase, ensuring created
/// prefixes by modules are in harmony.
#[derive(ModuleInfo, Clone)]
#[cfg_attr(feature = "native", derive(sov_modules_api::ModuleCallJsonSchema))]
pub struct Ibc<C: Context, Da: DaSpec> {
    #[address]
    pub address: C::Address,

    #[module]
    chain_state: sov_chain_state::ChainState<C, Da>,

    #[module]
    transfer: IbcTransfer<C>,

    // ----------- IBC core client state maps -------------
    #[state]
    client_counter: StateValue<u64>,

    #[state]
    client_state_map: StateMap<ClientId, AnyClientState, ProtobufCodec<Any>>,

    #[state]
    consensus_state_map: StateMap<ClientConsensusStatePath, AnyConsensusState, ProtobufCodec<Any>>,

    #[state]
    client_update_heights_vec: StateVec<Height>,

    #[state]
    client_update_host_times_map: StateMap<(ClientId, Height), Timestamp>,

    #[state]
    client_update_host_heights_map: StateMap<(ClientId, Height), Height>,

    // ----------- IBC core connection state maps -------------
    #[state]
    connection_counter: StateValue<u64>,

    #[state]
    connection_end_map: StateMap<ConnectionPath, ConnectionEnd, ProtobufCodec<RawConnectionEnd>>,

    #[state]
    client_connections_map: StateMap<ClientConnectionPath, Vec<ConnectionId>>,

    // ----------- IBC core channel state maps -------------
    #[state]
    channel_counter: StateValue<u64>,

    #[state]
    channel_end_map: StateMap<ChannelEndPath, ChannelEnd, ProtobufCodec<RawChannelEnd>>,

    #[state]
    send_sequence_map: StateMap<SeqSendPath, Sequence>,

    #[state]
    recv_sequence_map: StateMap<SeqRecvPath, Sequence>,

    #[state]
    ack_sequence_map: StateMap<SeqAckPath, Sequence>,

    #[state]
    packet_commitment_vec: StateVec<CommitmentPath>,

    #[state]
    packet_commitment_map: StateMap<CommitmentPath, PacketCommitment, PacketCommitmentCodec>,

    #[state]
    packet_receipt_vec: StateVec<ReceiptPath>,

    #[state]
    packet_receipt_map: StateMap<ReceiptPath, Receipt>,

    #[state]
    packet_ack_vec: StateVec<AckPath>,

    #[state]
    packet_ack_map: StateMap<AckPath, AcknowledgementCommitment, AcknowledgementCommitmentCodec>,
}

impl<C: Context, Da: DaSpec> sov_modules_api::Module for Ibc<C, Da> {
    type Context = C;

    type Config = ExampleModuleConfig;

    type CallMessage = call::CallMessage<C>;

    fn genesis(&self, config: &Self::Config, working_set: &mut WorkingSet<C>) -> Result<(), Error> {
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
                Ok(self.process_core_message(msg_envelope, context.clone(), working_set)?)
            }
            call::CallMessage::Transfer(sdk_token_transfer) => {
                Ok(self.transfer(sdk_token_transfer, context.clone(), working_set)?)
            }
        }
    }
}
