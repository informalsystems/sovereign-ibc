use async_trait::async_trait;
use ibc_client_tendermint::types::Header;
use ibc_core::channel::types::proto::v1::QueryPacketCommitmentRequest;
use ibc_core::client::types::proto::v1::{QueryClientStateRequest, QueryConsensusStateRequest};
use ibc_core::client::types::Height;
use ibc_core::handler::types::events::IbcEvent;
use ibc_core::host::types::path::{ClientConsensusStatePath, Path};
use ibc_core::host::ValidationContext;
use sov_celestia_client::types::client_message::test_util::dummy_sov_header;
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
    <Da as DaService>::Spec: Clone,
    P: MerkleProofSpec + Clone + 'static,
    <P as MerkleProofSpec>::Hasher: Send + Sync,
{
    type Message = RuntimeCall<S, Da::Spec>;

    async fn query(&self, request: QueryReq) -> QueryResp {
        info!("rollup: querying app with {:?}", request);

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
            QueryReq::Header(target_height, trusted_height) => {
                let blocks = self.da_core.blocks();

                let revision_height = target_height.revision_height() as usize;

                if revision_height > blocks.len() {
                    panic!("block index out of bounds");
                }

                let target_block = blocks[revision_height - 1].clone();

                let header = Header {
                    signed_header: target_block.signed_header,
                    validator_set: target_block.validators,
                    trusted_height,
                    trusted_next_validator_set: target_block.next_validators,
                };

                let sov_header = dummy_sov_header(
                    header,
                    Height::new(0, 1).unwrap(),
                    Height::new(0, revision_height as u64).unwrap(),
                );

                QueryResp::Header(sov_header.into())
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
        info!("rollup: submitting messages {:?}", msg);

        self.mempool.acquire_mutex().extend(msg);

        wait_for_block().await;

        vec![]
    }
}
