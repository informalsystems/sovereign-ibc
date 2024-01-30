//! Contains the Cosmos-specific message builders for the relayer.

use std::str::FromStr;

use base64::engine::general_purpose;
use base64::Engine;
use basecoin_app::modules::ibc::AnyClientState;
use ibc_app_transfer::types::msgs::transfer::MsgTransfer;
use ibc_app_transfer::types::packet::PacketData;
use ibc_app_transfer::types::{Coin, Memo, PrefixedDenom};
use ibc_core::channel::types::msgs::MsgRecvPacket;
use ibc_core::channel::types::packet::Packet;
use ibc_core::channel::types::timeout::TimeoutHeight;
use ibc_core::client::context::client_state::ClientStateCommon;
use ibc_core::client::types::msgs::{MsgCreateClient, MsgUpdateClient};
use ibc_core::client::types::Height;
use ibc_core::commitment_types::commitment::CommitmentProofBytes;
use ibc_core::host::types::identifiers::{ChannelId, PortId};
use ibc_core::host::types::path::{CommitmentPath, Path, SeqSendPath};
use ibc_core::primitives::proto::Any;
use ibc_core::primitives::{Signer, Timestamp, ToProto};
use sov_ibc::context::HOST_REVISION_NUMBER;

use crate::configs::TransferTestConfig;
use crate::relayer::handle::{Handle, QueryReq, QueryResp};
use crate::relayer::relay::MockRelayer;
use crate::sovereign::dummy_sov_client_state;

impl<SrcChain, DstChain> MockRelayer<SrcChain, DstChain>
where
    SrcChain: Handle,
    DstChain: Handle,
{
    /// Builds a create client message of type `Any`
    pub async fn build_msg_create_client_for_cos(&self) -> Any {
        let current_height = match self.src_chain_ctx().query(QueryReq::HostHeight).await {
            QueryResp::HostHeight(height) => height,
            _ => panic!("unexpected query response"),
        };

        let chain_id = match self.src_chain_ctx().query(QueryReq::ChainId).await {
            QueryResp::ChainId(chain_id) => chain_id,
            _ => panic!("unexpected query response"),
        };

        let sov_client_state = dummy_sov_client_state(chain_id, current_height);

        let consensus_state = match self
            .src_chain_ctx()
            .query(QueryReq::HostConsensusState(current_height))
            .await
        {
            QueryResp::HostConsensusState(cons) => cons,
            _ => panic!("unexpected query response"),
        };

        let msg_create_client = MsgCreateClient {
            client_state: sov_client_state.into(),
            consensus_state,
            signer: self.src_chain_ctx().signer().clone(),
        };

        msg_create_client.to_any()
    }

    /// Builds an update client message of type `Any`
    pub async fn build_msg_update_client_for_cos(&self, target_height: Height) -> Any {
        let client_id = self.src_client_id().clone();

        let any_client_state = match self
            .dst_chain_ctx()
            .query(QueryReq::ClientState(client_id.clone()))
            .await
        {
            QueryResp::ClientState(state) => state,
            _ => panic!("unexpected query response"),
        };

        let client_state = AnyClientState::try_from(any_client_state).unwrap();

        let header = match self
            .src_chain_ctx()
            .query(QueryReq::Header(
                target_height,
                client_state.latest_height(),
            ))
            .await
        {
            QueryResp::Header(header) => header,
            _ => panic!("unexpected query response"),
        };

        MsgUpdateClient {
            client_id,
            client_message: header,
            signer: self.dst_chain_ctx().signer().clone(),
        }
        .to_any()
    }

    /// Builds a Cosmos chain token transfer message; serialized to Any
    pub async fn build_msg_transfer_for_cos(&self, config: &TransferTestConfig) -> MsgTransfer {
        let memo = match config.sov_token_address {
            Some(token_address) => {
                let mut token_address_buf = String::new();

                general_purpose::STANDARD_NO_PAD
                    .encode_string(token_address, &mut token_address_buf);

                token_address_buf.into()
            }
            None => Memo::from_str("").unwrap(),
        };

        let packet_data = PacketData {
            token: Coin {
                denom: PrefixedDenom::from_str(&config.cos_denom).unwrap(),
                amount: config.amount.into(),
            },
            sender: Signer::from(config.cos_address.clone()),
            receiver: Signer::from(config.sov_address.to_string()),
            memo,
        };

        MsgTransfer {
            port_id_on_a: PortId::transfer(),
            chan_id_on_a: ChannelId::default(),
            packet_data,
            // NOTE: packet timeout height and timeout timestamp cannot both be 0
            timeout_height_on_b: TimeoutHeight::At(Height::new(HOST_REVISION_NUMBER, 200).unwrap()),
            timeout_timestamp_on_b: Timestamp::none(),
        }
    }

    /// Builds a receive packet message of type `Any`
    pub async fn build_msg_recv_packet_for_cos(
        &self,
        proof_height_on_a: Height,
        msg_transfer: MsgTransfer,
    ) -> MsgRecvPacket {
        let seq_send_path =
            SeqSendPath::new(&msg_transfer.port_id_on_a, &msg_transfer.chan_id_on_a);

        let resp = self
            .src_chain_ctx()
            .query(QueryReq::NextSeqSend(seq_send_path.clone()))
            .await;

        let next_seq_send = match resp {
            QueryResp::NextSeqSend(seq) => seq,
            _ => panic!("unexpected query response"),
        };

        let latest_seq_send = (u64::from(next_seq_send) - 1).into();

        let commitment_path = CommitmentPath::new(
            &msg_transfer.port_id_on_a,
            &msg_transfer.chan_id_on_a,
            latest_seq_send,
        );

        let resp = self
            .src_chain_ctx()
            .query(QueryReq::ValueWithProof(
                Path::Commitment(commitment_path.clone()),
                proof_height_on_a,
            ))
            .await;

        let (_, proof_bytes) = match resp {
            QueryResp::ValueWithProof(value, proof) => (value, proof),
            _ => panic!("unexpected query response"),
        };

        let proof_commitment_on_a = CommitmentProofBytes::try_from(proof_bytes).unwrap();

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

        MsgRecvPacket {
            packet,
            proof_commitment_on_a,
            proof_height_on_a,
            signer: self.dst_chain_ctx().signer().clone(),
        }
    }
}
