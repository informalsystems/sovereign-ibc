use std::time::Duration;

use ibc::clients::ics07_tendermint::client_state::{AllowUpdate, ClientState};
use ibc::core::ics24_host::identifier::ChainId;
use ibc::Height;

use ibc::core::ics23_commitment::specs::ProofSpecs;
use ibc::test_utils::get_dummy_bech32_account;
use ibc::Signer;
use ibc_proto::ics23::ProofSpec as RawProofSpec;

pub fn basecoin_proofspecs() -> ProofSpecs {
    let spec = RawProofSpec {
        leaf_spec: Some(ibc_proto::ics23::LeafOp {
            hash: ibc_proto::ics23::HashOp::Sha256.into(),
            prehash_key: ibc_proto::ics23::HashOp::NoHash.into(),
            prehash_value: ibc_proto::ics23::HashOp::NoHash.into(),
            length: ibc_proto::ics23::LengthOp::NoPrefix.into(),
            prefix: [0; 64].into(),
        }),
        inner_spec: Some(ibc_proto::ics23::InnerSpec {
            child_order: [0, 1, 2].into(),
            child_size: 32,
            min_prefix_length: 0,
            max_prefix_length: 64,
            empty_child: [0, 32].into(),
            hash: ibc_proto::ics23::HashOp::Sha256.into(),
        }),
        max_depth: 0,
        min_depth: 0,
        prehash_key_before_comparison: false,
    };
    [spec.clone(), spec].to_vec().into()
}

pub fn dummy_tm_client_state(chain_id: ChainId, latest_height: Height) -> ClientState {
    ClientState::new(
        chain_id,
        Default::default(),
        Duration::from_secs(64000),
        Duration::from_secs(128000),
        Duration::from_millis(3000),
        latest_height,
        basecoin_proofspecs(),
        Default::default(),
        AllowUpdate {
            after_expiry: false,
            after_misbehaviour: false,
        },
    )
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
      get_dummy_bech32_account(): {
        "basecoin": "0x1000000000",
        "othercoin": "0x1000000000",
        "samoleans": "0x1000000000"
      }
    })
}

pub fn dummy_signer() -> Signer {
    Signer::from("cosmos000000000000000000000000000000000000000".to_string())
}
