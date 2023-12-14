use async_trait::async_trait;
use ibc_core::channel::types::proto::v1::QueryPacketCommitmentRequest;
use ibc_core::client::types::proto::v1::{QueryClientStateRequest, QueryConsensusStateRequest};
use ibc_core::handler::types::events::IbcEvent;
use ibc_core::host::types::path::{ClientConsensusStatePath, Path};
use ibc_core::host::ValidationContext;
use sov_modules_api::{Context, WorkingSet};
use sov_rollup_interface::services::da::DaService;
use sov_state::{MerkleProofSpec, ProverStorage};
use tracing::info;

use crate::relayer::handle::{Handle, QueryReq, QueryResp};
use crate::setup::wait_for_block;
use crate::sovereign::{MockRollup, RuntimeCall};

#[async_trait]
impl<C, Da, S> Handle for MockRollup<C, Da, S>
where
    C: Context<Storage = ProverStorage<S>> + Send + Sync,
    Da: DaService<Error = anyhow::Error> + Clone,
    <Da as DaService>::Spec: Clone,
    S: MerkleProofSpec + Clone + 'static,
    <S as MerkleProofSpec>::Hasher: Send + Sync,
{
    type Message = RuntimeCall<C, Da::Spec>;

    async fn query(&self, request: QueryReq) -> QueryResp {
        info!("rollup: got query request: {:?}", request);

        let mut working_set = WorkingSet::new(self.prover_storage());

        let ibc_ctx = self.ibc_ctx(&mut working_set);

        match request {
            QueryReq::ChainId => QueryResp::ChainId(self.chain_id().clone()),
            QueryReq::ClientCounter => QueryResp::ClientCounter(ibc_ctx.client_counter().unwrap()),
            QueryReq::HostHeight => QueryResp::HostHeight(ibc_ctx.host_height().unwrap()),
            QueryReq::HostConsensusState(height) => {
                QueryResp::HostConsensusState(ibc_ctx.host_consensus_state(&height).unwrap().into())
            }
            QueryReq::ClientState(client_id) => {
                QueryResp::ClientState(ibc_ctx.client_state(&client_id).unwrap().into())
            }
            QueryReq::ConsensusState(client_id, height) => QueryResp::ConsensusState(
                ibc_ctx
                    .consensus_state(&ClientConsensusStatePath::new(
                        client_id,
                        height.revision_number(),
                        height.revision_height(),
                    ))
                    .unwrap()
                    .into(),
            ),
            QueryReq::Header(_, _) => {
                unimplemented!()
            }
            QueryReq::NextSeqSend(path) => {
                QueryResp::NextSeqSend(ibc_ctx.get_next_sequence_send(&path).unwrap())
            }
            QueryReq::ValueWithProof(path, _) => match path {
                Path::ClientState(path) => {
                    let req = QueryClientStateRequest {
                        client_id: path.0.to_string(),
                    };

                    let resp = self
                        .runtime()
                        .ibc
                        .client_state(req, &mut working_set)
                        .unwrap();

                    QueryResp::ValueWithProof(resp.client_state.unwrap().value, resp.proof)
                }
                Path::ClientConsensusState(path) => {
                    let req = QueryConsensusStateRequest {
                        client_id: path.client_id.to_string(),
                        revision_number: path.revision_number,
                        revision_height: path.revision_height,
                        latest_height: true,
                    };

                    let resp = self
                        .runtime()
                        .ibc
                        .consensus_state(req, &mut working_set)
                        .unwrap();

                    QueryResp::ValueWithProof(resp.consensus_state.unwrap().value, resp.proof)
                }

                Path::Commitment(path) => {
                    let req = QueryPacketCommitmentRequest {
                        port_id: path.port_id.to_string(),
                        channel_id: path.channel_id.to_string(),
                        sequence: path.sequence.into(),
                    };

                    let resp = self
                        .runtime()
                        .ibc
                        .packet_commitment(req, &mut working_set)
                        .unwrap();

                    QueryResp::ValueWithProof(resp.commitment, resp.proof)
                }
                _ => panic!("not implemented"),
            },
        }
    }

    async fn submit_msgs(&self, msg: Vec<Self::Message>) -> Vec<IbcEvent> {
        self.mempool.lock().unwrap().extend(msg);

        wait_for_block().await;

        vec![]
    }
}
