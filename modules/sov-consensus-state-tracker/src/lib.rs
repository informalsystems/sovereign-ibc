#![forbid(unsafe_code)]
#![deny(
    warnings,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms,
    clippy::unwrap_used
)]

use ibc_core::client::types::Height;
#[cfg(feature = "mock-da")]
use ibc_core::commitment_types::commitment::CommitmentRoot;
#[cfg(feature = "mock-da")]
use sov_celestia_client::types::consensus_state::{
    ConsensusState as SovConsensusState, TmConsensusParams,
};
use sov_ibc::clients::AnyConsensusState;
use sov_ibc::context::HOST_REVISION_NUMBER;
#[cfg(feature = "mock-da")]
use sov_mock_da::MockDaSpec;
use sov_modules_api::runtime::capabilities::{BatchSelector, Kernel, KernelSlotHooks};
use sov_modules_api::{Context, DaSpec, KernelModule, KernelWorkingSet, StateMapAccessor};
use sov_modules_core::kernel_state::BootstrapWorkingSet;
use sov_modules_core::Storage;
#[cfg(feature = "mock-da")]
use sov_rollup_interface::da::BlockHeaderTrait;

pub trait HasConsensusState: DaSpec {
    fn consensus_state(header: &Self::BlockHeader) -> AnyConsensusState;
}

#[cfg(feature = "mock-da")]
impl HasConsensusState for MockDaSpec {
    fn consensus_state(header: &Self::BlockHeader) -> AnyConsensusState {
        // Implement HasConsensusState for all DaSpecs that you wish to support,
        // and extract the consensus state from the header.
        // Suggestion: maybe add feature gates to this crate for each DaSpec.
        AnyConsensusState::Sovereign(
            SovConsensusState {
                root: CommitmentRoot::from_bytes(header.hash().as_ref()),
                da_params: TmConsensusParams::new(
                    tendermint::Time::from_unix_timestamp(
                        header.time.secs(),
                        header.time.subsec_nanos(),
                    )
                    .expect("time is valid"),
                    tendermint::Hash::None, // TODO: use `None` for the MockDaSpec, but later we use the real value when working with real DaSpecs
                ),
            }
            .into(),
        )
    }
}

#[derive(Clone)]
pub struct ConsensusStateTracker<K, C: Context, Da: DaSpec + HasConsensusState> {
    inner: K,
    ibc_module: sov_ibc::Ibc<C, Da>,
}

impl<K, C, Da> Default for ConsensusStateTracker<K, C, Da>
where
    K: Default,
    C: Context,
    Da: DaSpec + HasConsensusState,
{
    fn default() -> Self {
        Self {
            inner: K::default(),
            ibc_module: Default::default(),
        }
    }
}

impl<K, C, Da> KernelModule for ConsensusStateTracker<K, C, Da>
where
    C: Context,
    Da: DaSpec + HasConsensusState,
{
    type Context = C;
    type Config = ();
}

impl<K, C, Da> BatchSelector<Da> for ConsensusStateTracker<K, C, Da>
where
    K: BatchSelector<Da>,
    C: Context,
    Da: DaSpec + HasConsensusState,
{
    type Batch = K::Batch;
    type Context = K::Context;

    fn get_batches_for_this_slot<'a, 'k, I>(
        &self,
        current_blobs: I,
        working_set: &mut KernelWorkingSet<'k, Self::Context>,
    ) -> anyhow::Result<Vec<(Self::Batch, Da::Address)>>
    where
        I: IntoIterator<Item = &'a mut Da::BlobTransaction>,
    {
        self.inner
            .get_batches_for_this_slot(current_blobs, working_set)
    }
}

impl<K, C, Da> Kernel<C, Da> for ConsensusStateTracker<K, C, Da>
where
    K: Kernel<C, Da>,
    C: Context,
    Da: DaSpec + HasConsensusState,
{
    type GenesisConfig = K::GenesisConfig;
    type GenesisPaths = K::GenesisPaths;

    fn genesis(
        &self,
        config: &Self::GenesisConfig,
        working_set: &mut KernelWorkingSet<'_, C>,
    ) -> Result<(), anyhow::Error> {
        self.inner.genesis(config, working_set)
    }

    fn true_slot_number(&self, working_set: &mut BootstrapWorkingSet<'_, C>) -> u64 {
        self.inner.true_slot_number(working_set)
    }

    fn visible_slot_number(&self, working_set: &mut BootstrapWorkingSet<'_, C>) -> u64 {
        self.inner.visible_slot_number(working_set)
    }
}

impl<K, C, Da> KernelSlotHooks<C, Da> for ConsensusStateTracker<K, C, Da>
where
    K: KernelSlotHooks<C, Da>,
    C: Context,
    Da: DaSpec + HasConsensusState,
    <C as sov_modules_api::Spec>::Storage: Storage,
{
    fn begin_slot_hook(
        &self,
        slot_header: &Da::BlockHeader,
        validity_condition: &Da::ValidityCondition,
        pre_state_root: &<<C as sov_modules_api::Spec>::Storage as Storage>::Root,
        working_set: &mut sov_modules_api::StateCheckpoint<Self::Context>,
    ) -> C::GasUnit {
        let kernel_working_set = KernelWorkingSet::from_kernel(&self.inner, working_set);
        let visible_height = kernel_working_set.virtual_slot();

        // Workaround for the fact that zero is not a valid height (No block produced and processed yet)
        if visible_height > 0 {
            let height = Height::new(HOST_REVISION_NUMBER, visible_height).expect("valid height");
            let consensus_state = Da::consensus_state(slot_header);
            self.ibc_module.host_consensus_state_map.set(
                &height,
                &consensus_state,
                kernel_working_set.inner,
            );
        }

        self.inner
            .begin_slot_hook(slot_header, validity_condition, pre_state_root, working_set)
    }

    fn end_slot_hook(
        &self,
        gas_used: &C::GasUnit,
        working_set: &mut sov_modules_api::StateCheckpoint<Self::Context>,
    ) {
        self.inner.end_slot_hook(gas_used, working_set)
    }
}
