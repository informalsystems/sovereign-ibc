use ibc_core::client::context::client_state::ClientStateCommon;
use ibc_core::host::ValidationContext;
use ibc_query::core::channel::{
    QueryChannelClientStateRequest, QueryChannelClientStateResponse,
    QueryChannelConsensusStateRequest, QueryChannelConsensusStateResponse, QueryChannelRequest,
    QueryChannelResponse, QueryNextSequenceReceiveRequest, QueryNextSequenceReceiveResponse,
    QueryPacketAcknowledgementRequest, QueryPacketAcknowledgementResponse,
    QueryPacketCommitmentRequest, QueryPacketCommitmentResponse, QueryPacketReceiptRequest,
    QueryPacketReceiptResponse,
};
use ibc_query::core::client::{
    IdentifiedClientState, QueryClientStateRequest, QueryClientStateResponse,
    QueryConsensusStateRequest, QueryConsensusStateResponse, QueryUpgradedClientStateRequest,
    QueryUpgradedClientStateResponse, QueryUpgradedConsensusStateRequest,
    QueryUpgradedConsensusStateResponse,
};
use ibc_query::core::connection::{
    QueryClientConnectionsRequest, QueryClientConnectionsResponse,
    QueryConnectionClientStateRequest, QueryConnectionClientStateResponse,
    QueryConnectionConsensusStateRequest, QueryConnectionConsensusStateResponse,
    QueryConnectionRequest, QueryConnectionResponse,
};
use jsonrpsee::core::RpcResult;
use sov_ibc_transfer::to_jsonrpsee_error;
use sov_modules_api::Spec;

use crate::context::IbcContext;
use crate::helpers::{WithProof, WithoutProof};

/// The implementation of the IBC RPC methods.
///
/// The context is already created with:
/// - The working set at the query height, if provided.
/// - The latest working set.
///
/// So we can ignore the `query_height` parameter.
impl<'a, S: Spec> IbcContext<'a, S> {
    pub(super) fn client_state_response(
        &self,
        request: QueryClientStateRequest,
    ) -> RpcResult<QueryClientStateResponse> {
        let proof_height = self.host_height().map_err(to_jsonrpsee_error)?;

        let (client_state, proof) = self.query_client_state::<WithProof>(&request.client_id)?;

        Ok(QueryClientStateResponse::new(
            client_state
                .ok_or_else(|| {
                    to_jsonrpsee_error(format!(
                        "Client state not found for client {:?}",
                        request.client_id
                    ))
                })?
                .into(),
            proof,
            proof_height,
        ))
    }

    pub(super) fn consensus_state_response(
        &self,
        request: QueryConsensusStateRequest,
    ) -> RpcResult<QueryConsensusStateResponse> {
        let proof_height = self.host_height().map_err(to_jsonrpsee_error)?;

        let consensus_height = request.consensus_height.ok_or_else(|| {
            to_jsonrpsee_error("Consensus height is required for querying consensus state")
        })?;

        let (consensus_state, proof) = self.query_client_consensus_state::<WithProof>(
            &request.client_id,
            consensus_height.revision_number(),
            consensus_height.revision_height(),
        )?;

        Ok(QueryConsensusStateResponse::new(
            consensus_state
                .ok_or_else(|| {
                    to_jsonrpsee_error(format!(
                        "Consensus state not found for client {:?} at height {:?}",
                        request.client_id, consensus_height
                    ))
                })?
                .into(),
            proof,
            proof_height,
        ))
    }

    pub(super) fn upgraded_client_state_response(
        &self,
        _request: QueryUpgradedClientStateRequest,
    ) -> RpcResult<QueryUpgradedClientStateResponse> {
        let proof_height = self.host_height().map_err(to_jsonrpsee_error)?;

        let (upgraded_client_state, proof) =
            self.query_upgraded_client_state::<WithProof>(proof_height.revision_height())?;

        Ok(QueryUpgradedClientStateResponse::new(
            upgraded_client_state
                .ok_or_else(|| {
                    to_jsonrpsee_error(format!(
                        "Upgraded client state not found at height {proof_height:?}"
                    ))
                })?
                .into(),
            proof,
            proof_height,
        ))
    }

    pub(super) fn upgraded_consensus_state_response(
        &self,
        _request: QueryUpgradedConsensusStateRequest,
    ) -> RpcResult<QueryUpgradedConsensusStateResponse> {
        let proof_height = self.host_height().map_err(to_jsonrpsee_error)?;

        let (upgraded_consensus_state, proof) =
            self.query_upgraded_consensus_state::<WithProof>(proof_height.revision_height())?;

        Ok(QueryUpgradedConsensusStateResponse::new(
            upgraded_consensus_state
                .ok_or_else(|| {
                    to_jsonrpsee_error(format!(
                        "Upgraded consensus state not found at height {proof_height:?}"
                    ))
                })?
                .into(),
            proof,
            proof_height,
        ))
    }

    pub(super) fn connection_response(
        &self,
        request: QueryConnectionRequest,
    ) -> RpcResult<QueryConnectionResponse> {
        let proof_height = self.host_height().map_err(to_jsonrpsee_error)?;

        let (connection_end, proof) =
            self.query_connection_end::<WithProof>(&request.connection_id)?;

        Ok(QueryConnectionResponse::new(
            connection_end.ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "Connection not found for connection id {:?}",
                    request.connection_id
                ))
            })?,
            proof,
            proof_height,
        ))
    }

    pub(super) fn client_connections_response(
        &self,
        request: QueryClientConnectionsRequest,
    ) -> RpcResult<QueryClientConnectionsResponse> {
        let proof_height = self.host_height().map_err(to_jsonrpsee_error)?;

        let (client_connections, proof) =
            self.query_client_connections::<WithProof>(&request.client_id)?;

        Ok(QueryClientConnectionsResponse::new(
            client_connections.ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "Client connections not found for client id {:?}",
                    request.client_id
                ))
            })?,
            proof,
            proof_height,
        ))
    }

    pub(super) fn connection_client_state_response(
        &self,
        request: QueryConnectionClientStateRequest,
    ) -> RpcResult<QueryConnectionClientStateResponse> {
        let proof_height = self.host_height().map_err(to_jsonrpsee_error)?;

        let connection_end = self
            .query_connection_end::<WithoutProof>(&request.connection_id)?
            .ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "Connection not found for connection id {:?}",
                    request.connection_id
                ))
            })?;

        let (client_state, proof) =
            self.query_client_state::<WithProof>(connection_end.client_id())?;

        Ok(QueryConnectionClientStateResponse::new(
            IdentifiedClientState::new(
                connection_end.client_id().clone(),
                client_state
                    .ok_or_else(|| {
                        to_jsonrpsee_error(format!(
                            "Client state not found for connection {:?}",
                            request.connection_id
                        ))
                    })?
                    .into(),
            ),
            proof,
            proof_height,
        ))
    }

    pub(super) fn connection_consensus_state_response(
        &self,
        request: QueryConnectionConsensusStateRequest,
    ) -> RpcResult<QueryConnectionConsensusStateResponse> {
        let proof_height = self.host_height().map_err(to_jsonrpsee_error)?;

        let connection_end = self
            .query_connection_end::<WithoutProof>(&request.connection_id)?
            .ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "Connection not found for connection id {:?}",
                    request.connection_id
                ))
            })?;

        let (consensus_state, proof) = self.query_client_consensus_state::<WithProof>(
            connection_end.client_id(),
            request.height.revision_number(),
            request.height.revision_height(),
        )?;

        Ok(QueryConnectionConsensusStateResponse::new(
            consensus_state
                .ok_or_else(|| {
                    to_jsonrpsee_error(format!(
                        "Consensus state not found for connection {:?}",
                        request.connection_id
                    ))
                })?
                .into(),
            connection_end.client_id().clone(),
            proof,
            proof_height,
        ))
    }

    pub(super) fn channel_response(
        &self,
        request: QueryChannelRequest,
    ) -> RpcResult<QueryChannelResponse> {
        let proof_height = self.host_height().map_err(to_jsonrpsee_error)?;

        let (channel_end, proof) =
            self.query_channel_end::<WithProof>(&request.port_id, &request.channel_id)?;

        Ok(QueryChannelResponse::new(
            channel_end.ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "Channel not found for port id {:?} and channel id {:?}",
                    request.port_id, request.channel_id
                ))
            })?,
            proof,
            proof_height,
        ))
    }

    pub(super) fn channel_client_state_response(
        &self,
        request: QueryChannelClientStateRequest,
    ) -> RpcResult<QueryChannelClientStateResponse> {
        let proof_height = self.host_height().map_err(to_jsonrpsee_error)?;

        let channel_end = self
            .query_channel_end::<WithoutProof>(&request.port_id, &request.channel_id)?
            .ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "Channel not found for port id {:?} and channel id {:?}",
                    request.port_id, request.channel_id
                ))
            })?;

        let connection_id = channel_end.connection_hops().first().ok_or_else(|| {
            to_jsonrpsee_error(format!(
                "ConnectionId not found for channel {:?}",
                request.channel_id
            ))
        })?;

        let connection_end: ibc_core::connection::types::ConnectionEnd = self
            .query_connection_end::<WithoutProof>(connection_id)?
            .ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "ConnectionEnd not found for channel {:?}",
                    request.channel_id
                ))
            })?;

        let (client_state, proof) =
            self.query_client_state::<WithProof>(connection_end.client_id())?;

        Ok(QueryChannelClientStateResponse::new(
            IdentifiedClientState::new(
                connection_end.client_id().clone(),
                client_state
                    .ok_or_else(|| {
                        to_jsonrpsee_error(format!(
                            "Client state not found for channel {:?}",
                            request.channel_id
                        ))
                    })?
                    .into(),
            ),
            proof,
            proof_height,
        ))
    }

    pub(super) fn channel_consensus_state_response(
        &self,
        request: QueryChannelConsensusStateRequest,
    ) -> RpcResult<QueryChannelConsensusStateResponse> {
        let proof_height = self.host_height().map_err(to_jsonrpsee_error)?;

        let channel_end = self
            .query_channel_end::<WithoutProof>(&request.port_id, &request.channel_id)?
            .ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "Channel not found for port id {:?} and channel id {:?}",
                    request.port_id, request.channel_id
                ))
            })?;

        let connection_id = channel_end.connection_hops().first().ok_or_else(|| {
            to_jsonrpsee_error(format!(
                "ConnectionId not found for channel {:?}",
                request.channel_id
            ))
        })?;

        let connection_end = self
            .query_connection_end::<WithoutProof>(connection_id)?
            .ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "ConnectionEnd not found for channel {:?}",
                    request.channel_id
                ))
            })?;

        let client_state = self
            .query_client_state::<WithoutProof>(connection_end.client_id())?
            .ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "Client state not found for channel {:?}",
                    request.channel_id
                ))
            })?;

        let client_latest_height = client_state.latest_height();

        let (consensus_state, proof) = self.query_client_consensus_state::<WithProof>(
            connection_end.client_id(),
            client_latest_height.revision_number(),
            client_latest_height.revision_height(),
        )?;

        Ok(QueryChannelConsensusStateResponse::new(
            consensus_state
                .ok_or_else(|| {
                    to_jsonrpsee_error(format!(
                        "Consensus state not found for channel {:?}",
                        request.channel_id
                    ))
                })?
                .into(),
            connection_end.client_id().clone(),
            proof,
            proof_height,
        ))
    }

    pub(super) fn packet_commitment_response(
        &self,
        request: QueryPacketCommitmentRequest,
    ) -> RpcResult<QueryPacketCommitmentResponse> {
        let proof_height = self.host_height().map_err(to_jsonrpsee_error)?;

        let (commitment, proof) = self.query_packet_commitment::<WithProof>(
            &request.port_id,
            &request.channel_id,
            request.sequence,
        )?;

        Ok(QueryPacketCommitmentResponse::new(
            commitment.ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "Packet commitment not found for port id {:?}, channel id {:?} and sequence {:?}",
                    request.port_id, request.channel_id, request.sequence
                ))
            })?,
            proof,
            proof_height,
        ))
    }

    pub(super) fn packet_receipt_response(
        &self,
        request: QueryPacketReceiptRequest,
    ) -> RpcResult<QueryPacketReceiptResponse> {
        let proof_height = self.host_height().map_err(to_jsonrpsee_error)?;

        let (receipt, proof) = self.query_packet_receipt::<WithProof>(
            &request.port_id,
            &request.channel_id,
            request.sequence,
        )?;

        // packet_receipt_map models a set using constant unit value.
        // when the key (doesn't) exists in the map,
        // the receipt is (not) present and returns a (non) membership proof
        Ok(QueryPacketReceiptResponse::new(
            receipt.is_some(),
            proof,
            proof_height,
        ))
    }

    pub(super) fn packet_acknowledgement_response(
        &self,
        request: QueryPacketAcknowledgementRequest,
    ) -> RpcResult<QueryPacketAcknowledgementResponse> {
        let proof_height = self.host_height().map_err(to_jsonrpsee_error)?;

        let (acknowledgement, proof) = self.query_packet_acknowledgement::<WithProof>(
            &request.port_id,
            &request.channel_id,
            request.sequence,
        )?;

        Ok(QueryPacketAcknowledgementResponse::new(
            acknowledgement.ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "Packet acknowledgement not found for port id {:?}, channel id {:?} and sequence {:?}",
                    request.port_id, request.channel_id, request.sequence
                ))
            })?,
            proof,
            proof_height,
        ))
    }

    pub(super) fn next_sequence_receive_response(
        &self,
        request: QueryNextSequenceReceiveRequest,
    ) -> RpcResult<QueryNextSequenceReceiveResponse> {
        let proof_height = self.host_height().map_err(to_jsonrpsee_error)?;

        let (sequence, proof) =
            self.query_recv_sequence::<WithProof>(&request.port_id, &request.channel_id)?;

        Ok(QueryNextSequenceReceiveResponse::new(
            sequence.ok_or_else(|| {
                to_jsonrpsee_error(format!(
                    "Next sequence receive not found for port id {:?} and channel id {:?}",
                    request.port_id, request.channel_id
                ))
            })?,
            proof,
            proof_height,
        ))
    }
}
