//! Contains rollup specific message builders for the relayer.
use std::str::FromStr;

use ibc_app_transfer::types::msgs::transfer::MsgTransfer;
use ibc_app_transfer::types::packet::PacketData;
use ibc_app_transfer::types::{Coin, PrefixedDenom};
use ibc_core::channel::types::msgs::MsgRecvPacket;
use ibc_core::channel::types::packet::Packet;
use ibc_core::channel::types::timeout::TimeoutHeight;
use ibc_core::client::context::client_state::ClientStateCommon;
use ibc_core::client::types::msgs::{MsgCreateClient, MsgUpdateClient};
use ibc_core::client::types::Height;
use ibc_core::commitment_types::commitment::CommitmentProofBytes;
use ibc_core::commitment_types::merkle::MerkleProof;
use ibc_core::host::types::identifiers::{ChannelId, PortId};
use ibc_core::host::types::path::{CommitmentPath, Path, SeqSendPath};
use ibc_core::primitives::{Signer, Timestamp, ToProto};
use sov_bank::{CallMessage as BankCallMessage, TokenConfig};
use sov_ibc::call::CallMessage;
use sov_ibc::clients::AnyClientState;
use sov_modules_api::Spec;

use crate::configs::TransferTestConfig;
use crate::cosmos::dummy_tm_client_state;
use crate::relayer::handle::{Handle, QueryReq, QueryResp};
use crate::relayer::relay::MockRelayer;
use crate::sovereign::{DEFAULT_INIT_BALANCE, DEFAULT_SALT};

impl<SrcChain, DstChain> MockRelayer<SrcChain, DstChain>
where
    SrcChain: Handle,
    DstChain: Handle,
{
    /// Builds a create client message wrapped in a `CallMessage`
    pub async fn build_msg_create_client_for_sov(&self) -> CallMessage {
        let current_height = match self.dst_chain_ctx().query(QueryReq::HostHeight).await {
            QueryResp::HostHeight(height) => height,
            _ => panic!("unexpected query response"),
        };

        let chain_id = match self.dst_chain_ctx().query(QueryReq::ChainId).await {
            QueryResp::ChainId(chain_id) => chain_id,
            _ => panic!("unexpected query response"),
        };

        let tm_client_state = dummy_tm_client_state(chain_id, current_height);

        let consensus_state = match self
            .dst_chain_ctx()
            .query(QueryReq::HostConsensusState(current_height))
            .await
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

    /// Builds an update client message wrapped in a `CallMessage`
    pub async fn build_msg_update_client_for_sov(&self, target_height: Height) -> CallMessage {
        let client_id = self.dst_client_id().clone();

        let any_client_state = match self
            .src_chain_ctx()
            .query(QueryReq::ClientState(client_id.clone()))
            .await
        {
            QueryResp::ClientState(state) => state,
            _ => panic!("unexpected query response"),
        };

        let client_state = AnyClientState::try_from(any_client_state).unwrap();

        let header = match self
            .dst_chain_ctx()
            .query(QueryReq::Header(
                target_height,
                client_state.latest_height(),
            ))
            .await
        {
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

    /// Builds a `MsgTransfer` with the given configuration
    ///
    /// Note: keep the amount lower than the initial balance of the sender address
    pub fn build_msg_transfer_for_sov(&self, config: &TransferTestConfig) -> MsgTransfer {
        let packet_data = PacketData {
            token: Coin {
                denom: PrefixedDenom::from_str(&config.sov_denom).unwrap(),
                amount: config.amount.into(),
            },
            sender: Signer::from(config.sov_address.to_string()),
            receiver: Signer::from(config.cos_address.clone()),
            memo: "".into(),
        };

        MsgTransfer {
            port_id_on_a: PortId::transfer(),
            chan_id_on_a: ChannelId::zero(),
            packet_data,
            timeout_height_on_b: TimeoutHeight::At(Height::new(1, 200).unwrap()),
            timeout_timestamp_on_b: Timestamp::none(),
        }
    }

    /// Builds a receive packet message wrapped in a `CallMessage`
    pub async fn build_msg_recv_packet_for_sov(
        &self,
        proof_height_on_a: Height,
        msg_transfer: MsgTransfer,
    ) -> CallMessage {
        let seq_send_path =
            SeqSendPath::new(&msg_transfer.port_id_on_a, &msg_transfer.chan_id_on_a);

        let next_seq_send = match self
            .dst_chain_ctx()
            .query(QueryReq::NextSeqSend(seq_send_path.clone()))
            .await
        {
            QueryResp::NextSeqSend(seq) => seq,
            _ => panic!("unexpected query response"),
        };

        let latest_seq_send = (u64::from(next_seq_send) - 1).into();

        let commitment_path = CommitmentPath::new(
            &msg_transfer.port_id_on_a,
            &msg_transfer.chan_id_on_a,
            latest_seq_send,
        );

        let (_, proof_bytes) = match self
            .dst_chain_ctx()
            .query(QueryReq::ValueWithProof(
                Path::Commitment(commitment_path.clone()),
                Some(proof_height_on_a),
            ))
            .await
        {
            QueryResp::ValueWithProof(value, proof) => (value, proof),
            _ => panic!("unexpected query response"),
        };

        let commitment_proofs = CommitmentProofBytes::try_from(proof_bytes).unwrap();

        let merkle_proofs = MerkleProof::try_from(&commitment_proofs).unwrap();

        assert_eq!(merkle_proofs.proofs.len(), 2);

        let packet = Packet {
            seq_on_a: latest_seq_send,
            chan_id_on_a: msg_transfer.chan_id_on_a,
            port_id_on_a: msg_transfer.port_id_on_a,
            chan_id_on_b: ChannelId::zero(),
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

    /// Creates a token with the given configuration
    pub fn build_msg_create_token<S: Spec>(&self, token: &TokenConfig<S>) -> BankCallMessage<S> {
        BankCallMessage::CreateToken {
            salt: DEFAULT_SALT,
            token_name: token.token_name.clone(),
            initial_balance: DEFAULT_INIT_BALANCE,
            minter_address: token.address_and_balances[0].0.clone(),
            authorized_minters: vec![token.address_and_balances[0].0.clone()],
        }
    }
}
