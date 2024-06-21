use std::str::FromStr;
use std::sync::{Arc, Mutex};

use ibc_core::client::types::Height;
use ibc_core::host::types::identifiers::ChainId;
use tendermint::{AppHash, Time};
use tendermint_testgen::light_block::TmLightBlock;
use tendermint_testgen::{Generator, Header, LightBlock, Validator};
use tokio::runtime::Runtime;
use tracing::debug;
use typed_builder::TypedBuilder;

use crate::utils::MutexUtil;

#[derive(Clone, TypedBuilder)]
pub struct MockTendermint {
    /// Chain runtime
    #[builder(default = Arc::new(Runtime::new().unwrap()))]
    runtime: Arc<Runtime>,
    /// Chain identifier
    #[builder(default = ChainId::from_str("mock-cosmos-0").expect("never fails"))]
    chain_id: ChainId,
    /// Chain validators
    #[builder(default = Arc::new(Mutex::new(vec![
        Validator::new("1").voting_power(40),
        Validator::new("2").voting_power(30),
        Validator::new("3").voting_power(30),
    ])))]
    validators: Arc<Mutex<Vec<Validator>>>,
    /// Chain blocks
    #[builder(default = Arc::new(Mutex::new(vec![])))]
    blocks: Arc<Mutex<Vec<TmLightBlock>>>,
}

impl MockTendermint {
    /// Creates a new mock tendermint chain with the given chain id and validators.
    pub fn new(runtime: Runtime, chain_id: ChainId, validators: Vec<Validator>) -> Self {
        Self {
            runtime: Arc::new(runtime),
            chain_id,
            validators: Arc::new(Mutex::new(validators)),
            blocks: Arc::new(Mutex::new(vec![])),
        }
    }

    /// Returns the chain's runtime.
    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
    }

    /// Returns the chain id.
    pub fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }

    /// Returns the list of the generated blocks.
    pub fn blocks(&self) -> Vec<TmLightBlock> {
        self.blocks.acquire_mutex().clone()
    }

    /// Returns the chain's height.
    pub fn height(&self) -> u64 {
        self.blocks().len() as u64
    }

    /// Returns the list of the chain's validators.
    pub fn validators(&self) -> Vec<Validator> {
        self.validators.acquire_mutex().clone()
    }

    /// Generates a new light block for with the given app hash and adds it to
    /// the chain.
    pub fn grow_blocks(&self, root_hash: Vec<u8>) {
        let app_hash = AppHash::try_from(root_hash).expect("invalid app hash");

        let next_height = self.blocks().len() as u64 + 1;

        let header = Header::new(&self.validators())
            .chain_id(&self.chain_id.to_string())
            .height(next_height)
            .time(Time::now())
            .next_validators(&self.validators())
            .app_hash(app_hash.clone());

        let light_block = LightBlock::new_default_with_header(header)
            .generate()
            .expect("failed to generate light block");

        debug!(
            "tendermint: growing blocks at height={} with chain_id={}",
            next_height, self.chain_id
        );

        self.blocks.acquire_mutex().push(light_block);
    }

    pub fn advance_da_block_up_to(&mut self, height: Height) {
        for _ in 0..height.revision_height() - 1 {
            self.grow_blocks(vec![0; 32]);
        }
    }
}
