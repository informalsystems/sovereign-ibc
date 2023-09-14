use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

use ibc::applications::transfer::Memo;
use ibc::clients::ics07_tendermint::client_type as tm_client_type;
use ibc::clients::ics07_tendermint::consensus_state::ConsensusState as TmConsensusState;
use ibc::clients::ics07_tendermint::header::Header;
use ibc::core::ics02_client::client_state::ClientStateCommon;
use ibc::core::ics02_client::msgs::create_client::MsgCreateClient;
use ibc::core::ics02_client::msgs::update_client::MsgUpdateClient;
use ibc::core::ics02_client::msgs::ClientMsg;
use ibc::core::ics02_client::ClientExecutionContext;
use ibc::core::ics03_connection::connection::{
    ConnectionEnd, Counterparty as ConnCounterparty, State as ConnectionState,
};
use ibc::core::ics03_connection::version::Version as ConnectionVersion;
use ibc::core::ics04_channel::channel::{
    ChannelEnd, Counterparty as ChanCounterparty, Order, State as ChannelState,
};
use ibc::core::ics04_channel::msgs::{MsgRecvPacket, PacketMsg};
use ibc::core::ics04_channel::packet::{Packet, Sequence};
use ibc::core::ics04_channel::timeout::TimeoutHeight;
use ibc::core::ics04_channel::Version as ChannelVersion;
use ibc::core::ics23_commitment::commitment::CommitmentProofBytes;
use ibc::core::ics24_host::identifier::{ChainId, ChannelId, ClientId, ConnectionId, PortId};
use ibc::core::ics24_host::path::{
    ChannelEndPath, ClientConsensusStatePath, ClientStatePath, ConnectionPath, SeqSendPath,
};
use ibc::core::timestamp::Timestamp;
use ibc::core::{ExecutionContext, MsgEnvelope, ValidationContext};
use ibc::{Height, Signer};
use sov_bank::Bank;
use sov_ibc_transfer::call::SDKTokenTransfer;
use sov_modules_api::{Context, Module};
use sov_state::WorkingSet;
use tendermint::{AppHash, Hash, Time};
use tendermint_testgen::{Generator, Header as TestgenHeader, LightBlock, Validator};

use super::dummy::dummy_tm_client_state;
use crate::call::CallMessage;
use crate::context::clients::{AnyClientState, AnyConsensusState};
use crate::context::IbcExecutionContext;
use crate::Ibc;

/// Defines test fixture structure to interact with the bank and ibc modules
pub struct TestApp<'a, C: Context> {
    chain_id: ChainId,
    sdk_ctx: C,
    bank: Bank<C>,
    ibc_ctx: IbcExecutionContext<'a, C>,
}

impl<'a, C: Context> TestApp<'a, C> {
    /// Initializes the test fixture
    pub fn new(
        chain_id: ChainId,
        sdk_ctx: C,
        bank: Bank<C>,
        ibc: Ibc<C>,
        working_set: &'a mut WorkingSet<C::Storage>,
    ) -> Self {
        let shared_working_set = Rc::new(RefCell::new(working_set));

        let ibc_execution_ctx = IbcExecutionContext {
            ibc,
            working_set: shared_working_set,
        };

        Self {
            chain_id,
            sdk_ctx,
            bank,
            ibc_ctx: ibc_execution_ctx,
        }
    }

    pub fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }

    /// Returns access to the ibc module
    pub fn working_set(&self) -> &Rc<RefCell<&'a mut WorkingSet<C::Storage>>> {
        &self.ibc_ctx.working_set
    }

    /// Returns access to the bank module
    pub fn bank(&self) -> &Bank<C> {
        &self.bank
    }

    pub fn ibc_ctx(&self) -> &IbcExecutionContext<'a, C> {
        &self.ibc_ctx
    }

    /// Returns access to the transfer module
    pub fn transfer(&self) -> &sov_ibc_transfer::Transfer<C> {
        &self.ibc_ctx.ibc.transfer
    }

    /// Establishes a tendermint light client on the ibc module
    pub fn setup_client(&mut self) {
        let client_counter = self.ibc_ctx.client_counter().unwrap();

        let client_id = ClientId::new(tm_client_type(), client_counter).unwrap();

        let client_state_path = ClientStatePath::new(&client_id);

        let client_state = AnyClientState::Tendermint(dummy_tm_client_state(
            self.chain_id.clone(),
            Height::new(0, 10).unwrap(),
        ));

        self.ibc_ctx
            .store_client_state(client_state_path, client_state)
            .unwrap();

        let consensus_state_path =
            ClientConsensusStatePath::new(&client_id, &Height::new(0, 10).unwrap());

        let consensus_state = AnyConsensusState::Tendermint(TmConsensusState::new(
            vec![].into(),
            Time::now(),
            Hash::None,
        ));

        self.ibc_ctx
            .store_consensus_state(consensus_state_path, consensus_state)
            .unwrap();
    }

    /// Establishes a connection on the ibc module with `Open` state
    pub fn setup_connection(&mut self) {
        let client_counter = self.ibc_ctx.client_counter().unwrap();

        let client_id = ClientId::new(tm_client_type(), client_counter).unwrap();

        let connection_id = ConnectionId::new(0);

        let connection_path = ConnectionPath::new(&connection_id);

        let prefix = self.ibc_ctx.commitment_prefix();

        let connection_end = ConnectionEnd::new(
            ConnectionState::Open,
            client_id.clone(),
            ConnCounterparty::new(client_id, Some(connection_id), prefix),
            vec![ConnectionVersion::default()],
            Default::default(),
        )
        .unwrap();

        self.ibc_ctx
            .store_connection(&connection_path, connection_end)
            .unwrap();
    }

    /// Establishes a channel on the ibc module with `Open` state
    pub fn setup_channel(&mut self) {
        let connection_id = ConnectionId::new(0);

        let channel_id = ChannelId::new(0);

        let port_id = PortId::transfer();

        let channel_end_path = ChannelEndPath::new(&port_id, &channel_id);

        let channel_end = ChannelEnd::new(
            ChannelState::Open,
            Order::default(),
            ChanCounterparty::new(PortId::transfer(), Some(channel_id.clone())),
            vec![connection_id],
            ChannelVersion::default(),
        )
        .unwrap();

        self.ibc_ctx
            .store_channel(&channel_end_path, channel_end)
            .unwrap();

        let seq_send_path = SeqSendPath::new(&port_id, &channel_id);

        let initial_seq = Sequence::from(0);

        self.ibc_ctx
            .store_next_sequence_send(&seq_send_path, initial_seq)
            .unwrap();
    }

    /// Submits an ibc message wrapped in a `CallMessage` to the ibc module
    pub fn send_ibc_message(&self, call_message: CallMessage<C>) {
        self.ibc_ctx
            .ibc
            .call(
                call_message,
                &self.sdk_ctx,
                *self.working_set().borrow_mut(),
            )
            .unwrap();
    }

    /// Builds a create client message wrapped in a `CallMessage`
    pub fn build_msg_create_client(&self) -> CallMessage<C> {
        let tm_client_state =
            dummy_tm_client_state(self.chain_id.clone(), Height::new(0, 10).unwrap());

        // Dummy tendermint consensus state
        let consensus_state = TmConsensusState::new(vec![0].into(), Time::now(), Hash::None);

        let msg_create_client = MsgCreateClient {
            client_state: tm_client_state.into(),
            consensus_state: consensus_state.into(),
            signer: Signer::from(self.sdk_ctx.sender().to_string()),
        };

        CallMessage::Core(MsgEnvelope::Client(ClientMsg::CreateClient(
            msg_create_client,
        )))
    }

    /// TODO: Builds an update client message wrapped in a `CallMessage`
    pub fn build_msg_update_client(&self) -> CallMessage<C> {
        let client_counter = self.ibc_ctx.client_counter().unwrap();

        let client_id = ClientId::new(tm_client_type(), client_counter).unwrap();

        let client_state = self.ibc_ctx.client_state(&client_id).unwrap();

        let validators = [Validator::new("1"), Validator::new("2")];

        let testgen_header = TestgenHeader::new(&validators)
            .chain_id(self.chain_id.as_str())
            .height(11)
            .time(Time::now())
            .next_validators(&validators)
            .app_hash(AppHash::default());

        let light_block = LightBlock::new_default_with_header(testgen_header)
            .generate()
            .unwrap();

        let header = Header {
            signed_header: light_block.signed_header,
            validator_set: light_block.validators,
            trusted_height: client_state.latest_height(),
            trusted_next_validator_set: light_block.next_validators,
        };

        let msg_update_client = MsgUpdateClient {
            client_id,
            client_message: header.into(),
            signer: Signer::from(self.sdk_ctx.sender().to_string()),
        };

        CallMessage::Core(MsgEnvelope::Client(ClientMsg::UpdateClient(
            msg_update_client,
        )))
    }

    /// Builds a sdk token transfer message wrapped in a `CallMessage` with the given amount
    /// Note: keep the amount value lower than the initial balance of the sender address
    pub fn build_sdk_transfer(
        &self,
        token: C::Address,
        sender: C::Address,
        receiver: C::Address,
        amount: u64,
    ) -> CallMessage<C> {
        let msg_transfer = SDKTokenTransfer {
            port_id_on_a: PortId::transfer(),
            chan_id_on_a: ChannelId::default(),
            timeout_height_on_b: TimeoutHeight::no_timeout(),
            timeout_timestamp_on_b: Timestamp::none(),
            token_address: token,
            amount,
            sender: Signer::from(sender.to_string()),
            receiver: Signer::from(receiver.to_string()),
            memo: Memo::from_str("").unwrap(),
        };

        CallMessage::Transfer(msg_transfer)
    }

    /// TODO: Builds a receive packet message wrapped in a `CallMessage` with the given amount
    pub fn build_recv_packet(&self) -> CallMessage<C> {
        let packet = Packet {
            seq_on_a: Sequence::from(0),
            port_id_on_a: PortId::transfer(),
            chan_id_on_b: ChannelId::default(),
            port_id_on_b: PortId::transfer(),
            chan_id_on_a: ChannelId::default(),
            data: vec![0],
            timeout_height_on_b: TimeoutHeight::no_timeout(),
            timeout_timestamp_on_b: Timestamp::none(),
        };

        let msg_recv_packet = MsgRecvPacket {
            packet,
            proof_commitment_on_a: CommitmentProofBytes::try_from(vec![1]).unwrap(),
            proof_height_on_a: Height::new(0, 10).unwrap(),
            signer: Signer::from(self.sdk_ctx.sender().to_string()),
        };

        CallMessage::Core(MsgEnvelope::Packet(PacketMsg::Recv(msg_recv_packet)))
    }
}
