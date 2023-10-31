use std::sync::Arc;

use ibc::clients::ics07_tendermint::client_type as tm_client_type;
use ibc::core::ics02_client::client_state::ClientStateCommon;
use ibc::core::ics02_client::msgs::create_client::MsgCreateClient;
use ibc::core::ics02_client::msgs::update_client::MsgUpdateClient;
use ibc::core::ics04_channel::msgs::MsgRecvPacket;
use ibc::core::ics04_channel::packet::Packet;
use ibc::core::ics04_channel::timeout::TimeoutHeight;
use ibc::core::ics24_host::identifier::{ChannelId, ClientId, PortId};
use ibc::core::ics24_host::path::{CommitmentPath, SeqSendPath};
use ibc::core::timestamp::Timestamp;
use ibc::core::{Msg, ValidationContext};
use ibc::{Height, Signer};
use ibc_query::core::context::ProvableContext;
use sov_ibc::call::CallMessage;
use sov_modules_api::default_context::DefaultContext;

use super::context::ChainContext;
use super::handle::Handle;
use crate::cosmos::helpers::dummy_tm_client_state;

/// The relay context for relaying between a mock sovereign chain and a mock
/// cosmos chain
#[derive(Clone)]
pub struct MockRelayer<SrcChain, DstChain>
where
    SrcChain: Handle,
    DstChain: Handle,
{
    pub src_chain_ctx: Arc<ChainContext<SrcChain>>,
    pub dst_chain_ctx: Arc<ChainContext<DstChain>>,
    pub src_client_id: ClientId,
    pub dst_client_id: ClientId,
}

impl<SrcChain, DstChain> MockRelayer<SrcChain, DstChain>
where
    SrcChain: Handle,
    DstChain: Handle,
{
    pub fn new(
        src_chain: Arc<SrcChain>,
        dst_chain: Arc<DstChain>,
        src_client_id: ClientId,
        dst_client_id: ClientId,
        src_address: Signer,
        dst_address: Signer,
    ) -> MockRelayer<SrcChain, DstChain> {
        let src_chain_ctx = Arc::new(ChainContext::new(src_chain, src_address));

        let dst_chain_ctx = Arc::new(ChainContext::new(dst_chain, dst_address));

        Self {
            src_chain_ctx,
            dst_chain_ctx,
            src_client_id,
            dst_client_id,
        }
    }

    pub fn src_chain_ctx(&self) -> &Arc<ChainContext<SrcChain>> {
        &self.src_chain_ctx
    }

    pub fn dst_chain_ctx(&self) -> &Arc<ChainContext<DstChain>> {
        &self.dst_chain_ctx
    }

    pub fn src_client_id(&self) -> &ClientId {
        &self.src_client_id
    }

    pub fn dst_client_id(&self) -> &ClientId {
        &self.dst_client_id
    }

    /// Builds a create client message wrapped in a `CallMessage`
    pub fn build_msg_create_client(&self) -> CallMessage<DefaultContext> {
        let current_height = self.dst_chain_ctx().query_ibc().host_height().unwrap();

        let tm_client_state = dummy_tm_client_state(
            self.dst_chain_ctx().query_chain_id().clone(),
            current_height,
        );

        let consensus_state = self
            .dst_chain_ctx()
            .query_ibc()
            .host_consensus_state(&current_height)
            .unwrap();

        let any_cons_state = self.dst_chain_ctx().consensus_state_to_any(consensus_state);

        let msg_create_client = MsgCreateClient {
            client_state: tm_client_state.into(),
            consensus_state: any_cons_state,
            signer: self.src_chain_ctx().signer().clone(),
        };

        CallMessage::Core(msg_create_client.to_any())
    }

    pub fn build_msg_update_client_for_sov(
        &self,
        target_height: Height,
    ) -> CallMessage<DefaultContext> {
        let client_counter = self
            .src_chain_ctx()
            .query_ibc()
            .client_counter()
            .unwrap()
            .checked_sub(1)
            .unwrap();

        let client_id = ClientId::new(tm_client_type(), client_counter).unwrap();

        let client_state = self
            .src_chain_ctx()
            .query_ibc()
            .client_state(&client_id)
            .unwrap();

        let header = self
            .dst_chain_ctx()
            .query_header(&target_height, &client_state.latest_height());

        let msg_update_client = MsgUpdateClient {
            client_id,
            client_message: header.into(),
            signer: self.src_chain_ctx().signer().clone(),
        };

        CallMessage::Core(msg_update_client.to_any())
    }

    pub fn build_msg_recv_packet_for_sov(
        &self,
        proof_height_on_a: Height,
    ) -> CallMessage<DefaultContext> {
        let seq_send_path = SeqSendPath::new(&PortId::transfer(), &ChannelId::default());

        let latest_seq_send = (u64::from(
            self.dst_chain_ctx()
                .query_ibc()
                .get_next_sequence_send(&seq_send_path)
                .expect("no error"),
        ) - 1)
            .into();

        let commitment_path =
            CommitmentPath::new(&seq_send_path.0, &seq_send_path.1, latest_seq_send);

        let packet_commitment_data = self
            .dst_chain_ctx()
            .query_ibc()
            .get_packet_commitment(&commitment_path)
            .expect("no error");

        let proof_commitment_on_a = self
            .dst_chain_ctx()
            .query_ibc()
            .get_proof(proof_height_on_a, &commitment_path.into())
            .expect("no error");

        let packet = Packet {
            seq_on_a: latest_seq_send,
            port_id_on_a: PortId::transfer(),
            chan_id_on_b: ChannelId::default(),
            port_id_on_b: PortId::transfer(),
            chan_id_on_a: ChannelId::default(),
            data: packet_commitment_data.into_vec(),
            timeout_height_on_b: TimeoutHeight::no_timeout(),
            timeout_timestamp_on_b: Timestamp::none(),
        };

        let msg_recv_packet = MsgRecvPacket {
            packet,
            proof_commitment_on_a: proof_commitment_on_a.try_into().expect("no error"),
            proof_height_on_a,
            signer: self.src_chain_ctx().signer().clone(),
        };

        CallMessage::Core(msg_recv_packet.to_any())
    }
}
