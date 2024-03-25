#![forbid(unsafe_code)]
#![deny(
    warnings,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms,
    clippy::unwrap_used
)]

#[cfg(feature = "celestia-da")]
mod celestia_da;
#[cfg(feature = "mock-da")]
mod mock_da;

#[cfg(feature = "celestia-da")]
pub use celestia_da::*;
use ibc_core::client::types::Height;
#[cfg(feature = "mock-da")]
pub use mock_da::*;
use sov_celestia_client::consensus_state::ConsensusState as HostConsensusState;
use sov_ibc::context::HOST_REVISION_NUMBER;
use sov_modules_api::runtime::capabilities::{BatchSelector, Kernel, KernelSlotHooks};
use sov_modules_api::{DaSpec, Gas, KernelModule, KernelWorkingSet, Spec, StateCheckpoint};
use sov_modules_core::kernel_state::BootstrapWorkingSet;
use sov_modules_core::Storage;

/// Implement HasConsensusState for all DaSpecs that you wish to support, and
/// extract the consensus state from the header.
pub trait HasConsensusState: DaSpec {
    fn consensus_state(header: &Self::BlockHeader) -> HostConsensusState;
}

#[derive(Clone)]
pub struct ConsensusStateTracker<K, S: Spec, Da: DaSpec + HasConsensusState> {
    inner: K,
    ibc: sov_ibc::Ibc<S>,
    da: core::marker::PhantomData<Da>,
}

impl<K, S, Da> Default for ConsensusStateTracker<K, S, Da>
where
    K: Default,
    S: Spec,
    Da: DaSpec + HasConsensusState,
{
    fn default() -> Self {
        Self {
            inner: K::default(),
            ibc: Default::default(),
            da: Default::default(),
        }
    }
}

impl<K, S, Da> KernelModule for ConsensusStateTracker<K, S, Da>
where
    S: Spec,
    Da: DaSpec + HasConsensusState,
{
    type Spec = S;
    type Config = ();
}

impl<K, S, Da> BatchSelector<Da> for ConsensusStateTracker<K, S, Da>
where
    K: BatchSelector<Da>,
    S: Spec,
    Da: DaSpec + HasConsensusState,
{
    type Batch = K::Batch;
    type Spec = K::Spec;

    fn get_batches_for_this_slot<'a, 'k, I>(
        &self,
        current_blobs: I,
        working_set: &mut KernelWorkingSet<'k, Self::Spec>,
    ) -> anyhow::Result<Vec<(Self::Batch, Da::Address)>>
    where
        I: IntoIterator<Item = &'a mut Da::BlobTransaction>,
    {
        self.inner
            .get_batches_for_this_slot(current_blobs, working_set)
    }
}

impl<K, S, Da> Kernel<S, Da> for ConsensusStateTracker<K, S, Da>
where
    K: Kernel<S, Da>,
    S: Spec,
    Da: DaSpec + HasConsensusState,
{
    type GenesisConfig = K::GenesisConfig;
    type GenesisPaths = K::GenesisPaths;

    fn genesis(
        &self,
        config: &Self::GenesisConfig,
        working_set: &mut KernelWorkingSet<'_, S>,
    ) -> Result<(), anyhow::Error> {
        self.inner.genesis(config, working_set)
    }

    fn true_slot_number(&self, working_set: &mut BootstrapWorkingSet<'_, S>) -> u64 {
        self.inner.true_slot_number(working_set)
    }

    fn visible_slot_number(&self, working_set: &mut BootstrapWorkingSet<'_, S>) -> u64 {
        self.inner.visible_slot_number(working_set)
    }
}

impl<K, S, Da> KernelSlotHooks<S, Da> for ConsensusStateTracker<K, S, Da>
where
    K: KernelSlotHooks<S, Da>,
    S: Spec,
    Da: DaSpec + HasConsensusState,
    <S as Spec>::Storage: Storage,
{
    fn begin_slot_hook(
        &self,
        slot_header: &Da::BlockHeader,
        validity_condition: &Da::ValidityCondition,
        pre_state_root: &<<Self::Spec as Spec>::Storage as Storage>::Root,
        working_set: &mut StateCheckpoint<Self::Spec>,
    ) -> <S::Gas as Gas>::Price {
        // NOTE: The `begin_slot_hook` is executed on the inner first to update
        // states within the basic kernel, such as the `sov-chain-state`,
        // ensuring that the current slot number remains current.
        let gas_price = self.inner.begin_slot_hook(
            slot_header,
            validity_condition,
            pre_state_root,
            working_set,
        );

        let kernel_working_set = KernelWorkingSet::from_kernel(&self.inner, working_set);

        let visible_slot_number = kernel_working_set.current_slot();

        // Workaround the fact that zero is not a valid height (No DA block produced and processed yet)
        if visible_slot_number > 0 {
            let height =
                Height::new(HOST_REVISION_NUMBER, visible_slot_number).expect("valid height");

            self.ibc
                .host_height_map
                .set(&height, kernel_working_set.inner);

            let consensus_state = Da::consensus_state(slot_header);

            self.ibc.host_timestamp_map.set(
                &consensus_state.timestamp().into(),
                kernel_working_set.inner,
            );

            self.ibc.host_consensus_state_map.set(
                &height,
                &consensus_state,
                kernel_working_set.inner,
            );
        }

        gas_price
    }

    fn end_slot_hook(&self, gas_used: &S::Gas, working_set: &mut StateCheckpoint<Self::Spec>) {
        self.inner.end_slot_hook(gas_used, working_set)
    }
}
