use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, from_json, Deps, MessageInfo};
use ibc_client_tendermint::types::Header;
use ibc_core::client::types::{Height, Status};
use ibc_core::host::types::identifiers::ChainId;
use sov_celestia_client::types::client_message::test_util::dummy_sov_header;
use sov_celestia_client::types::client_state::test_util::{
    dummy_checksum, dummy_sov_client_state, dummy_sov_consensus_state,
};
use tendermint_testgen::Generator;

use crate::entrypoints::{instantiate, query, sudo};
use crate::types::{
    ContractResult, ExportMetadataMsg, GenesisMetadata, InstantiateMsg, QueryMsg, QueryResponse,
    StatusMsg, SudoMsg, UpdateStateMsgRaw, VerifyClientMessageRaw,
};

pub fn dummy_msg_info() -> MessageInfo {
    mock_info("creator", &coins(1000, "ibc"))
}

pub fn dummy_instantiate_msg(latest_height: Height) -> InstantiateMsg {
    let sov_client_state = dummy_sov_client_state(ChainId::new("rollup").unwrap(), latest_height);

    let sov_consensus_state = dummy_sov_consensus_state();

    InstantiateMsg {
        client_state: sov_client_state.encode_thru_any(),
        consensus_state: sov_consensus_state.encode_thru_any(),
        checksum: dummy_checksum(),
    }
}

pub fn dummy_client_message(trusted_height: Height, target_height: Height) -> Vec<u8> {
    let validators = vec![
        tendermint_testgen::Validator::new("1"),
        tendermint_testgen::Validator::new("2"),
    ];

    let header =
        tendermint_testgen::Header::new(&validators).height(target_height.revision_height());

    let light_block = tendermint_testgen::LightBlock::new_default_with_header(header)
        .generate()
        .expect("failed to generate light block");

    let tm_header = Header {
        signed_header: light_block.signed_header,
        validator_set: light_block.validators,
        trusted_height,
        trusted_next_validator_set: light_block.next_validators,
    };

    let sov_header = dummy_sov_header(tm_header, trusted_height, target_height);

    sov_header.encode_thru_any()
}

pub fn verify_client_message(deps: Deps<'_>, client_message: Vec<u8>) {
    let query_msg = QueryMsg::VerifyClientMessage(VerifyClientMessageRaw { client_message });

    let resp_bytes = query(deps, mock_env(), query_msg).unwrap();

    let resp: QueryResponse = from_json(resp_bytes).unwrap();

    assert!(resp.is_valid);
    assert!(resp.status.is_none());
    assert!(resp.found_misbehaviour.is_none());
}

pub fn check_client_status(deps: Deps<'_>, expected: Status) {
    let query_msg = QueryMsg::Status(StatusMsg {});

    let resp_bytes = query(deps, mock_env(), query_msg).unwrap();

    let resp: QueryResponse = from_json(resp_bytes).unwrap();

    assert_eq!(resp.status.unwrap(), expected.to_string());
}

pub fn _get_metadata(deps: Deps<'_>) -> Option<Vec<GenesisMetadata>> {
    let query_msg = QueryMsg::ExportMetadata(ExportMetadataMsg {});

    let resp_bytes = query(deps, mock_env(), query_msg).unwrap();

    let resp: QueryResponse = from_json(resp_bytes).unwrap();

    resp.genesis_metadata
}

#[test]
fn happy_cw_create_client() {
    let mut deps = mock_dependencies();

    let latest_height = Height::new(0, 5).unwrap();

    let instantiate_msg = dummy_instantiate_msg(latest_height);

    let resp = instantiate(deps.as_mut(), mock_env(), dummy_msg_info(), instantiate_msg).unwrap();

    assert_eq!(0, resp.messages.len());

    let contract_result: ContractResult = from_json(resp.data.unwrap()).unwrap();

    assert!(contract_result.heights.is_none());

    check_client_status(deps.as_ref(), Status::Active);
}

#[test]
fn happy_cw_update_client() {
    let mut deps = mock_dependencies();

    let initial_state_height = Height::new(0, 5).unwrap();

    let final_state_height = Height::new(0, 10).unwrap();

    let instantiate_msg = dummy_instantiate_msg(initial_state_height);

    instantiate(deps.as_mut(), mock_env(), dummy_msg_info(), instantiate_msg).unwrap();

    let client_message_bytes = dummy_client_message(initial_state_height, final_state_height);

    verify_client_message(deps.as_ref(), client_message_bytes.clone());

    let resp = sudo(
        deps.as_mut(),
        mock_env(),
        SudoMsg::UpdateState(UpdateStateMsgRaw {
            client_message: client_message_bytes,
        }),
    )
    .unwrap();

    assert_eq!(0, resp.messages.len());

    let contract_result: ContractResult = from_json(resp.data.unwrap()).unwrap();

    assert_eq!(contract_result.heights.unwrap(), vec![final_state_height]);

    check_client_status(deps.as_ref(), Status::Active);
}
