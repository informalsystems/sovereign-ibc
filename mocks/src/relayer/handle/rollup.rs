use async_trait::async_trait;
use ibc_core::client::context::ClientValidationContext;
use ibc_core::client::types::Height;
use ibc_core::handler::types::events::IbcEvent;
use ibc_core::host::types::path::{ClientConsensusStatePath, Path};
use ibc_core::host::ValidationContext;
use ibc_query::core::channel::QueryPacketCommitmentRequest;
use ibc_query::core::client::{QueryClientStateRequest, QueryConsensusStateRequest};
use sov_consensus_state_tracker::HasConsensusState;
use sov_modules_api::{Spec, WorkingSet};
use sov_rollup_interface::services::da::DaService;
use sov_state::{MerkleProofSpec, ProverStorage};
use tracing::info;

use crate::relayer::handle::{Handle, QueryReq, QueryResp};
use crate::sovereign::{MockRollup, RuntimeCall};
use crate::utils::{wait_for_block, MutexUtil};

#[async_trait]
impl<S, Da, P> Handle for MockRollup<S, Da, P>
where
    S: Spec<Storage = ProverStorage<P>> + Send + Sync,
    Da: DaService<Error = anyhow::Error> + Clone,
    Da::Spec: HasConsensusState,
    <Da as DaService>::Spec: Clone,
    P: MerkleProofSpec + Clone + 'static,
    <P as MerkleProofSpec>::Hasher: Send + Sync,
{
    type Message = RuntimeCall<S>;

    async fn query(&self, request: QueryReq) -> QueryResp {
        info!("rollup: querying app with {:?}", request);

        let mut working_set = WorkingSet::new(self.prover_storage());

        let ibc_ctx: sov_ibc::context::IbcContext<'_, S> = self.ibc_ctx(&mut working_set);

        match request {
            QueryReq::ChainId => QueryResp::ChainId(self.chain_id().clone()),
            QueryReq::ClientCounter => QueryResp::ClientCounter(ibc_ctx.client_counter().unwrap()),
            QueryReq::HostHeight => QueryResp::HostHeight(ibc_ctx.host_height().unwrap()),
            QueryReq::HostConsensusState(height) => {
                QueryResp::HostConsensusState(ibc_ctx.host_consensus_state(&height).unwrap().into())
            }
            QueryReq::ClientState(client_id) => QueryResp::ClientState(
                ibc_ctx
                    .get_client_validation_context()
                    .client_state(&client_id)
                    .unwrap()
                    .into(),
            ),
            QueryReq::ConsensusState(client_id, height) => QueryResp::ConsensusState(
                ibc_ctx
                    .get_client_validation_context()
                    .consensus_state(&ClientConsensusStatePath::new(
                        client_id,
                        height.revision_number(),
                        height.revision_height(),
                    ))
                    .unwrap()
                    .into(),
            ),
            QueryReq::Header(target_height, trusted_height) => {
                let sov_header = self.obtain_ibc_header(target_height, trusted_height);

                QueryResp::Header(sov_header.into())
            }
            QueryReq::NextSeqSend(path) => {
                QueryResp::NextSeqSend(ibc_ctx.get_next_sequence_send(&path).unwrap())
            }
            QueryReq::ValueWithProof(path, h) => match path {
                Path::ClientState(path) => {
                    let req = QueryClientStateRequest {
                        client_id: path.0,
                        query_height: h,
                    };

                    let resp = self
                        .runtime()
                        .ibc
                        .client_state(req, &mut working_set)
                        .unwrap();

                    QueryResp::ValueWithProof(resp.client_state.value, resp.proof)
                }
                Path::ClientConsensusState(path) => {
                    let req = QueryConsensusStateRequest {
                        client_id: path.client_id,
                        consensus_height: Some(
                            Height::new(path.revision_number, path.revision_height)
                                .expect("Never fails"),
                        ),
                        query_height: h,
                    };

                    let resp = self
                        .runtime()
                        .ibc
                        .consensus_state(req, &mut working_set)
                        .unwrap();

                    QueryResp::ValueWithProof(resp.consensus_state.value, resp.proof)
                }
                Path::Commitment(path) => {
                    let req = QueryPacketCommitmentRequest {
                        port_id: path.port_id,
                        channel_id: path.channel_id,
                        sequence: path.sequence,
                        query_height: h,
                    };

                    let resp = self
                        .runtime()
                        .ibc
                        .packet_commitment(req, &mut working_set)
                        .unwrap();

                    QueryResp::ValueWithProof(resp.packet_commitment.into_vec(), resp.proof)
                }
                _ => panic!("not implemented"),
            },
        }
    }

    async fn submit_msgs(&self, msg: Vec<Self::Message>) -> Vec<IbcEvent> {
        info!("rollup: submitting messages {:?}", msg);

        self.mempool.acquire_mutex().extend(msg);

        wait_for_block().await;

        vec![]
    }
}
