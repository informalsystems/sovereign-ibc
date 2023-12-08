use std::str::FromStr;
use std::sync::Arc;

use ibc_app_transfer::types::msgs::transfer::MsgTransfer;
use ibc_app_transfer::types::packet::PacketData;
use ibc_app_transfer::types::{Coin, Memo, PrefixedCoin, PrefixedDenom};
use ibc_client_tendermint::types::client_type as tm_client_type;
use ibc_core::channel::types::msgs::MsgRecvPacket;
use ibc_core::channel::types::packet::Packet;
use ibc_core::channel::types::timeout::TimeoutHeight;
use ibc_core::client::context::client_state::ClientStateCommon;
use ibc_core::client::types::msgs::{MsgCreateClient, MsgUpdateClient};
use ibc_core::client::types::Height;
use ibc_core::commitment_types::commitment::CommitmentProofBytes;
use ibc_core::commitment_types::merkle::MerkleProof;
use ibc_core::commitment_types::proto::ics23::CommitmentProof;
use ibc_core::commitment_types::proto::v1::MerkleProof as RawMerkleProof;
use ibc_core::host::types::identifiers::{ChannelId, ClientId, PortId};
use ibc_core::host::types::path::{CommitmentPath, Path, SeqSendPath};
use ibc_core::primitives::proto::Any;
use ibc_core::primitives::{Msg, Signer, Timestamp};
use prost::Message;
use sov_ibc::call::CallMessage;
use sov_ibc::clients::AnyClientState;
use sov_ibc_transfer::call::SDKTokenTransfer;
use sov_modules_api::default_context::DefaultContext;
use sov_modules_api::{Context, Spec};

use super::context::ChainContext;
use super::handle::{Handle, QueryReq, QueryResp};
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
    pub fn build_msg_create_client_for_sov(&self) -> CallMessage<DefaultContext> {
        let current_height = match self.dst_chain_ctx().query(QueryReq::HostHeight) {
            QueryResp::HostHeight(height) => height,
            _ => panic!("unexpected query response"),
        };

        let chain_id = match self.dst_chain_ctx().query(QueryReq::ChainId) {
            QueryResp::ChainId(chain_id) => chain_id,
            _ => panic!("unexpected query response"),
        };

        let tm_client_state = dummy_tm_client_state(chain_id, current_height);

        let consensus_state = match self
            .dst_chain_ctx()
            .query(QueryReq::HostConsensusState(current_height))
        {
            QueryResp::HostConsensusState(cons) => cons,
            _ => panic!("unexpected query response"),
        };

        let msg_create_client = MsgCreateClient {
            client_state: tm_client_state.into(),
            consensus_state,
            signer: self.src_chain_ctx().signer().clone(),
        };

        CallMessage::Core(msg_create_client.to_any())
    }

    /// Builds a create client message of type `Any`
    pub fn build_msg_create_client_for_cos(&self) -> Any {
        let current_height = match self.src_chain_ctx().query(QueryReq::HostHeight) {
            QueryResp::HostHeight(height) => height,
            _ => panic!("unexpected query response"),
        };

        let chain_id = match self.src_chain_ctx().query(QueryReq::ChainId) {
            QueryResp::ChainId(chain_id) => chain_id,
            _ => panic!("unexpected query response"),
        };

        let tm_client_state = dummy_tm_client_state(chain_id, current_height);

        let consensus_state = match self
            .src_chain_ctx()
            .query(QueryReq::HostConsensusState(current_height))
        {
            QueryResp::HostConsensusState(cons) => cons,
            _ => panic!("unexpected query response"),
        };

        let msg_create_client = MsgCreateClient {
            client_state: tm_client_state.into(),
            consensus_state,
            signer: self.src_chain_ctx().signer().clone(),
        };

        msg_create_client.to_any()
    }

    /// Builds an update client message wrapped in a `CallMessage`
    pub fn build_msg_update_client_for_sov(
        &self,
        target_height: Height,
    ) -> CallMessage<DefaultContext> {
        let client_counter = match self.src_chain_ctx().query(QueryReq::ClientCounter) {
            QueryResp::ClientCounter(counter) => counter,
            _ => panic!("unexpected query response"),
        }
        .checked_sub(1)
        .unwrap();

        let client_id = ClientId::new(tm_client_type(), client_counter).unwrap();

        let any_client_state = match self
            .src_chain_ctx()
            .query(QueryReq::ClientState(client_id.clone()))
        {
            QueryResp::ClientState(state) => state,
            _ => panic!("unexpected query response"),
        };

        let client_state = AnyClientState::try_from(any_client_state).unwrap();

        let header = match self.dst_chain_ctx().query(QueryReq::Header(
            target_height,
            client_state.latest_height(),
        )) {
            QueryResp::Header(header) => header,
            _ => panic!("unexpected query response"),
        };

        let msg_update_client = MsgUpdateClient {
            client_id,
            client_message: header,
            signer: self.src_chain_ctx().signer().clone(),
        };

        CallMessage::Core(msg_update_client.to_any())
    }

    /// Builds an update client message of type `Any`
    pub fn build_msg_update_client_for_cos(&self, target_height: Height) -> Any {
        let client_counter = match self.dst_chain_ctx().query(QueryReq::ClientCounter) {
            QueryResp::ClientCounter(counter) => counter,
            _ => panic!("unexpected query response"),
        };

        let client_id = ClientId::new(tm_client_type(), client_counter).unwrap();

        let any_client_state = match self
            .dst_chain_ctx()
            .query(QueryReq::ClientState(client_id.clone()))
        {
            QueryResp::ClientState(state) => state,
            _ => panic!("unexpected query response"),
        };

        let client_state = AnyClientState::try_from(any_client_state).unwrap();

        let header = match self.src_chain_ctx().query(QueryReq::Header(
            target_height,
            client_state.latest_height(),
        )) {
            QueryResp::Header(header) => header,
            _ => panic!("unexpected query response"),
        };

        let msg_update_client = MsgUpdateClient {
            client_id,
            client_message: header,
            signer: self.dst_chain_ctx().signer().clone(),
        };

        msg_update_client.to_any()
    }

    /// Builds a sdk token transfer message wrapped in a `CallMessage` with the given amount
    /// Note: keep the amount value lower than the initial balance of the sender address
    pub fn build_sdk_transfer_for_sov<C: Context>(
        &self,
        token: <C as Spec>::Address,
        sender: Signer,
        receiver: Signer,
        amount: u64,
    ) -> CallMessage<C> {
        let msg_transfer = SDKTokenTransfer {
            port_id_on_a: PortId::transfer(),
            chan_id_on_a: ChannelId::default(),
            timeout_height_on_b: TimeoutHeight::At(Height::new(1, 200).unwrap()),
            timeout_timestamp_on_b: Timestamp::none(),
            token_address: token,
            amount,
            sender,
            receiver,
            memo: Memo::from_str("").unwrap(),
        };

        CallMessage::Transfer(msg_transfer)
    }

    /// Builds a receive packet message wrapped in a `CallMessage`
    pub fn build_msg_recv_packet_for_sov<C: Context>(
        &self,
        proof_height_on_a: Height,
        msg_transfer: MsgTransfer,
    ) -> CallMessage<C> {
        let seq_send_path = SeqSendPath::new(&PortId::transfer(), &ChannelId::default());

        let next_seq_send = match self
            .dst_chain_ctx()
            .query(QueryReq::NextSeqSend(seq_send_path.clone()))
        {
            QueryResp::NextSeqSend(seq) => seq,
            _ => panic!("unexpected query response"),
        };

        let latest_seq_send = (u64::from(next_seq_send) - 1).into();

        let commitment_path =
            CommitmentPath::new(&seq_send_path.0, &seq_send_path.1, latest_seq_send);

        let (_, proof_bytes) = match self.dst_chain_ctx().query(QueryReq::ValueWithProof(
            Path::Commitment(commitment_path.clone()),
            proof_height_on_a,
        )) {
            QueryResp::ValueWithProof(value, proof) => (value, proof),
            _ => panic!("unexpected query response"),
        };

        let commitment_proofs = CommitmentProofBytes::try_from(proof_bytes).unwrap();

        let merkle_proofs = MerkleProof::from(RawMerkleProof::try_from(commitment_proofs).unwrap());

        assert_eq!(merkle_proofs.proofs.len(), 2);

        let packet = Packet {
            seq_on_a: latest_seq_send,
            chan_id_on_a: msg_transfer.chan_id_on_a,
            port_id_on_a: msg_transfer.port_id_on_a,
            chan_id_on_b: ChannelId::default(),
            port_id_on_b: PortId::transfer(),
            data: serde_json::to_vec(&msg_transfer.packet_data).unwrap(),
            timeout_height_on_b: msg_transfer.timeout_height_on_b,
            timeout_timestamp_on_b: msg_transfer.timeout_timestamp_on_b,
        };

        let msg_recv_packet = MsgRecvPacket {
            packet,
            proof_commitment_on_a: merkle_proofs.try_into().expect("no error"),
            proof_height_on_a,
            signer: self.src_chain_ctx().signer().clone(),
        };

        CallMessage::Core(msg_recv_packet.to_any())
    }

    /// Builds a Cosmos chain token transfer message; serialized to Any
    pub fn build_msg_transfer_for_cos(
        &self,
        denom: &str,
        sender: Signer,
        receiver: Signer,
        amount: u64,
    ) -> MsgTransfer {
        let packet_data = PacketData {
            token: Coin {
                denom: PrefixedDenom::from_str(denom).unwrap(),
                amount: amount.into(),
            },
            sender,
            receiver,
            memo: Memo::from_str("").unwrap(),
        };

        MsgTransfer {
            port_id_on_a: PortId::transfer(),
            chan_id_on_a: ChannelId::default(),
            packet_data,
            // NOTE: packet timeout height and timeout timestamp cannot both be 0
            // Sovereign chain's initial revision number has been set to 1
            timeout_height_on_b: TimeoutHeight::At(Height::new(1, 200).unwrap()),
            timeout_timestamp_on_b: Timestamp::none(),
        }
    }

    /// Builds a receive packet message of type `Any`
    pub fn build_msg_recv_packet_for_cos<C: Context>(
        &self,
        proof_height_on_a: Height,
        sdk_transfer: SDKTokenTransfer<C>,
    ) -> Any {
        let seq_send_path = SeqSendPath::new(&PortId::transfer(), &ChannelId::default());

        let resp = self
            .src_chain_ctx()
            .query(QueryReq::NextSeqSend(seq_send_path.clone()));

        let next_seq_send = match resp {
            QueryResp::NextSeqSend(seq) => seq,
            _ => panic!("unexpected query response"),
        };

        let latest_seq_send = (u64::from(next_seq_send) - 1).into();

        let commitment_path =
            CommitmentPath::new(&seq_send_path.0, &seq_send_path.1, latest_seq_send);

        let resp = self.src_chain_ctx().query(QueryReq::ValueWithProof(
            Path::Commitment(commitment_path.clone()),
            proof_height_on_a,
        ));

        let (_, proof_bytes) = match resp {
            QueryResp::ValueWithProof(value, proof) => (value, proof),
            _ => panic!("unexpected query response"),
        };

        let commitment_proofs = CommitmentProofBytes::try_from(proof_bytes).unwrap();

        let merkle_proofs = MerkleProof::from(RawMerkleProof::try_from(commitment_proofs).unwrap());

        let resp = self.dst_chain_ctx().query(QueryReq::ValueWithProof(
            Path::Commitment(commitment_path.clone()),
            proof_height_on_a,
        ));

        let (_, proof_bytes) = match resp {
            QueryResp::ValueWithProof(value, proof) => (value, proof),
            _ => panic!("unexpected query response"),
        };

        let proof_commitment_on_a =
            CommitmentProof::decode(proof_bytes.as_slice()).expect("no error");

        assert_eq!(merkle_proofs.proofs[0], proof_commitment_on_a);
        assert_eq!(merkle_proofs.proofs.len(), 2);

        let packet_data = PacketData {
            token: PrefixedCoin {
                denom: sdk_transfer.token_address.to_string().parse().unwrap(),
                amount: sdk_transfer.amount.into(),
            },
            sender: sdk_transfer.sender,
            receiver: sdk_transfer.receiver,
            memo: sdk_transfer.memo,
        };

        let packet = Packet {
            seq_on_a: latest_seq_send,
            chan_id_on_a: sdk_transfer.chan_id_on_a,
            port_id_on_a: sdk_transfer.port_id_on_a,
            chan_id_on_b: ChannelId::default(),
            port_id_on_b: PortId::transfer(),
            data: serde_json::to_vec(&packet_data).unwrap(),
            timeout_height_on_b: sdk_transfer.timeout_height_on_b,
            timeout_timestamp_on_b: sdk_transfer.timeout_timestamp_on_b,
        };

        let msg_recv_packet = MsgRecvPacket {
            packet,
            proof_commitment_on_a: merkle_proofs.try_into().expect("no error"),
            proof_height_on_a,
            signer: self.dst_chain_ctx().signer().clone(),
        };

        msg_recv_packet.to_any()
    }
}
