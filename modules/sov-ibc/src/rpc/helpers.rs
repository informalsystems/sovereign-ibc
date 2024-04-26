use borsh::BorshSerialize;
use ibc_core::client::types::Height;
use jsonrpsee::core::RpcResult;
use sov_ibc_transfer::to_jsonrpsee_error;
use sov_modules_api::{Spec, StateMap, WorkingSet};
use sov_state::storage::{StateCodec, StateItemCodec};

use crate::context::IbcContext;
use crate::Ibc;

impl<S: Spec> Ibc<S> {
    pub(super) fn handle_request<F, Response>(
        &self,
        request_height: Option<Height>,
        working_set: &mut WorkingSet<S>,
        method: F,
    ) -> Response
    where
        F: FnOnce(&IbcContext<'_, S>) -> Response,
    {
        match request_height {
            Some(h) => {
                // note: IbcContext doesn't take ownership of the archival_working_set.
                // So it can't be returned from the this method. Instead, a closure is
                // passed to the method and the result is returned from the closure.
                let mut archival_working_set = working_set.get_archival_at(h.revision_height());
                let ibc_ctx = IbcContext::new(self, &mut archival_working_set);
                method(&ibc_ctx)
            }
            None => {
                let ibc_ctx = IbcContext::new(self, working_set);
                method(&ibc_ctx)
            }
        }
    }
}

/// Trait for proof agnostic storage value retrieval.
///
/// This trait is introduced to avoid duplication in the query methods
/// that fetch values with and without proofs -
/// [`WithProof`] and [`WithoutProof`] respectively.
///
/// This trait allows to have a single query methods for both cases, e.g.:
/// ```ignore
/// let (client_state, proof) = ibc_ctx.query_client_state::<WithProof>(client_id)?;
/// let client_state = ibc_ctx.query_client_state::<WithoutProof>(client_id)?;
/// ```
/// Although the [`WithProof`] case is only required for user-facing
/// query services, such as RPC, we generalized it to avoid code duplication.
pub trait StorageValue {
    type Output<V>;
    fn value_at_key<K, V, C, S>(
        key: &K,
        state_map: &StateMap<K, V, C>,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<Self::Output<V>>
    where
        S: Spec,
        C: StateCodec,
        <C as StateCodec>::ValueCodec: StateItemCodec<V>,
        <C as StateCodec>::KeyCodec: StateItemCodec<K>;
}

/// Implementation of [`StorageValue`] for values without proofs.
pub struct WithoutProof;

impl StorageValue for WithoutProof {
    type Output<V> = Option<V>;

    fn value_at_key<K, V, C, S>(
        key: &K,
        state_map: &StateMap<K, V, C>,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<Self::Output<V>>
    where
        S: Spec,
        C: StateCodec,
        <C as StateCodec>::ValueCodec: StateItemCodec<V>,
        <C as StateCodec>::KeyCodec: StateItemCodec<K>,
    {
        Ok(state_map.get(key, working_set))
    }
}

/// Implementation of [`StorageValue`] for values with proofs.
pub struct WithProof;

impl StorageValue for WithProof {
    type Output<V> = (Option<V>, Vec<u8>);

    fn value_at_key<K, V, C, S>(
        key: &K,
        state_map: &StateMap<K, V, C>,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<Self::Output<V>>
    where
        S: Spec,
        C: StateCodec,
        <C as StateCodec>::ValueCodec: StateItemCodec<V>,
        <C as StateCodec>::KeyCodec: StateItemCodec<K>,
    {
        let result = state_map.get_with_proof(key, working_set);

        Ok((
            result
                .value
                .map(|bytes| state_map.codec().value_codec().try_decode(bytes.value()))
                .transpose()
                .map_err(|e| to_jsonrpsee_error(format!("{e:?}")))?,
            result.proof.try_to_vec().map_err(to_jsonrpsee_error)?,
        ))
    }
}
