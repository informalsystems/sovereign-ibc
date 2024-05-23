use basecoin::store::avl::get_proof_spec;
use ibc_client_tendermint::client_state::ClientState;
use ibc_client_tendermint::types::ConsensusState as TmConsensusState;
use ibc_core::client::types::Height;
use ibc_core::commitment_types::specs::ProofSpecs;
use ibc_core::host::types::identifiers::ChainId;
use ibc_core::primitives::Signer;
use ibc_testkit::fixtures::clients::tendermint::ClientStateConfig;
use ibc_testkit::fixtures::core::signer::dummy_bech32_account;
use tendermint::{Hash, Time};
pub fn basecoin_proof_specs() -> ProofSpecs {
    [get_proof_spec(), get_proof_spec()]
        .to_vec()
        .try_into()
        .expect("should convert successfully")
}

pub fn dummy_tm_client_state(chain_id: ChainId, latest_height: Height) -> ClientState {
    ClientStateConfig::builder()
        .proof_specs(basecoin_proof_specs())
        .build()
        .into_client_state(chain_id, latest_height)
        .unwrap()
}

pub fn dummy_tm_consensus_state() -> TmConsensusState {
    TmConsensusState::new(
        vec![0].into(),
        Time::now(),
        // Hash for default validator set
        Hash::Sha256([
            0xd6, 0xb9, 0x39, 0x22, 0xc3, 0x3a, 0xae, 0xbe, 0xc9, 0x4, 0x35, 0x66, 0xcb, 0x4b,
            0x1b, 0x48, 0x36, 0x5b, 0x13, 0x58, 0xb6, 0x7c, 0x7d, 0xef, 0x98, 0x6d, 0x9e, 0xe1,
            0x86, 0x1b, 0xc1, 0x43,
        ]),
    )
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
