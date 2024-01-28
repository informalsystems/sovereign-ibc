use basecoin_store::avl::get_proof_spec;
use ibc_client_tendermint::client_state::ClientState;
use ibc_core::client::types::Height;
use ibc_core::commitment_types::specs::ProofSpecs;
use ibc_core::host::types::identifiers::ChainId;
use ibc_core::primitives::Signer;
use ibc_testkit::fixtures::clients::tendermint::ClientStateConfig;
use ibc_testkit::fixtures::core::signer::dummy_bech32_account;

pub fn basecoin_proof_specs() -> ProofSpecs {
    [get_proof_spec(), get_proof_spec()].to_vec().into()
}

pub fn dummy_tm_client_state(chain_id: ChainId, current_height: Height) -> ClientState {
    ClientStateConfig::builder()
        .chain_id(chain_id)
        .latest_height(current_height)
        .build()
        .try_into()
        .unwrap()
}

pub fn genesis_app_state() -> serde_json::Value {
    serde_json::json!({
      "cosmos12xpmzmfpf7tn57xg93rne2hc2q26lcfql5efws": {
        "basecoin": "0x1000000000",
        "othercoin": "0x1000000000",
        "samoleans": "0x1000000000"
      },
      "cosmos1t2e0nyjhwn3revunvf2uperhftvhzu4euuzva9": {
        "basecoin": "0x250",
        "othercoin": "0x5000"
      },
      "cosmos1uawm90a5xm36kjmaazv89nxmfr8s8cyzkjqytd": {
        "acidcoin": "0x500"
      },
      "cosmos1ny9epydqnr7ymqhmgfvlshp3485cuqlmt7vsmf": {},
      "cosmos1xwgdxu4ahd9eevtfnq5f7w4td3rqnph4llnngw": {
        "acidcoin": "0x500",
        "basecoin": "0x0",
        "othercoin": "0x100"
      },
      "cosmos1mac8xqhun2c3y0njptdmmh3vy8nfjmtm6vua9u": {
        "basecoin": "0x1000"
      },
      "cosmos1wkvwnez6fkjn63xaz7nzpm4zxcd9cetqmyh2y8": {
        "basecoin": "0x1"
      },
      "cosmos166vcha998g7tl8j8cq0kwa8rfvm68cqmj88cff": {
        "basecoin": "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
      },
      "cosmos1as9ap057eellftptlhyw5ajna7uaeewzkk6fnz": {
        "basecoin": "0x1000000000"
      },
      dummy_bech32_account(): {
        "basecoin": "0x1000000000",
        "othercoin": "0x1000000000",
        "samoleans": "0x1000000000"
      }
    })
}

pub fn dummy_signer() -> Signer {
    Signer::from("cosmos000000000000000000000000000000000000000".to_string())
}
