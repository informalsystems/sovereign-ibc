use std::time::Duration;

use borsh::BorshDeserialize;
use ibc_client_tendermint::types::proto::v1::{
    ClientState as RawClientState, ConsensusState as RawConsensusState,
};
use ibc_client_tendermint::types::{ClientState, ConsensusState};
use ibc_core::client::context::client_state::ClientStateCommon;
use ibc_core::client::types::Height;
use ibc_core::host::types::path::{ClientConsensusStatePath, ClientStatePath, Path};
use ibc_core::primitives::proto::Protobuf;
use jmt::proof::SparseMerkleProof;
use sha2::Sha256;
use sov_ibc::clients::AnyClientState;
use tokio::time::sleep;

use crate::relayer::handle::{Handle, QueryReq, QueryResp};
use crate::setup::setup;

#[tokio::test]
async fn test_create_client_on_sov() {
    let (rly, mut rollup) = setup(false).await;

    let msg_create_client = rly.build_msg_create_client_for_sov();

    rollup.apply_msg(vec![msg_create_client]).await;

    let client_counter = match rly.src_chain_ctx().query(QueryReq::ClientCounter) {
        QueryResp::ClientCounter(counter) => counter,
        _ => panic!("Unexpected response"),
    };

    assert_eq!(client_counter, 1);

    match rly.src_chain_ctx().query(QueryReq::ValueWithProof(
        Path::ClientState(ClientStatePath(rly.src_client_id().clone())),
        Height::new(0, 4).unwrap(),
    )) {
        QueryResp::ValueWithProof(value, proof) => {
            let _: ClientState = Protobuf::<RawClientState>::decode(&mut value.as_slice()).unwrap();
            SparseMerkleProof::<Sha256>::deserialize(&mut proof.as_slice()).unwrap();
        }
        _ => panic!("unexpected response"),
    }

    match rly.src_chain_ctx().query(QueryReq::ValueWithProof(
        Path::ClientConsensusState(ClientConsensusStatePath {
            client_id: rly.src_client_id().clone(),
            revision_number: 0,
            revision_height: 4,
        }),
        Height::new(0, 4).unwrap(),
    )) {
        QueryResp::ValueWithProof(value, proof) => {
            let _: ConsensusState =
                Protobuf::<RawConsensusState>::decode(&mut value.as_slice()).unwrap();
            SparseMerkleProof::<Sha256>::deserialize(&mut proof.as_slice()).unwrap();
        }
        _ => panic!("unexpected response"),
    }
}

#[tokio::test]
async fn test_update_client_on_sov() {
    let (rly, mut rollup) = setup(false).await;

    let msg_create_client = rly.build_msg_create_client_for_sov();

    rollup.apply_msg(vec![msg_create_client]).await;

    // Waits for the mock cosmos chain to progress a few blocks
    sleep(Duration::from_secs(1)).await;

    let target_height = match rly.dst_chain_ctx().query(QueryReq::HostHeight) {
        QueryResp::HostHeight(height) => height,
        _ => panic!("unexpected response"),
    };

    let msg_update_client = rly.build_msg_update_client_for_sov(target_height);

    rollup.apply_msg(vec![msg_update_client]).await;

    let any_client_state = match rly
        .src_chain_ctx()
        .query(QueryReq::ClientState(rly.src_client_id().clone()))
    {
        QueryResp::ClientState(state) => state,
        _ => panic!("unexpected response"),
    };

    let client_state = AnyClientState::try_from(any_client_state).unwrap();

    assert_eq!(client_state.latest_height(), target_height);
}

#[tokio::test]
async fn test_create_client_on_cos() {
    let (rly, _) = setup(false).await;

    let msg_create_client = rly.build_msg_create_client_for_cos();

    rly.dst_chain_ctx().send_msg(vec![msg_create_client]);

    // Waits for the mock cosmos chain to commit the block
    sleep(Duration::from_secs(1)).await;

    let _client_counter = match rly.dst_chain_ctx().query(QueryReq::ClientCounter) {
        QueryResp::ClientCounter(counter) => counter,
        _ => panic!("Unexpected response"),
    };

    let client_state = match rly
        .dst_chain_ctx()
        .query(QueryReq::ClientState(rly.dst_client_id().clone()))
    {
        QueryResp::ClientState(state) => state,
        _ => panic!("unexpected response"),
    };

    let client_state = AnyClientState::try_from(client_state).unwrap();

    let _consensus_state = match rly.dst_chain_ctx().query(QueryReq::ConsensusState(
        rly.dst_client_id().clone(),
        client_state.latest_height(),
    )) {
        QueryResp::ConsensusState(state) => state,
        _ => panic!("unexpected response"),
    };
}
