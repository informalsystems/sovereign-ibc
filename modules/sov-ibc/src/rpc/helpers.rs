use std::cell::RefCell;
use std::rc::Rc;

use borsh::BorshSerialize;
use ibc_core::client::types::Height;
use ibc_core::host::ValidationContext;
use jsonrpsee::core::RpcResult;
use jsonrpsee::types::ErrorObjectOwned;
use sov_modules_api::{Spec, StateMap, WorkingSet};
use sov_state::storage::{StateCodec, StateItemCodec};

use crate::context::IbcContext;
use crate::Ibc;

impl<S: Spec> Ibc<S> {
    /// Determines the query height to use for the given request. If the query
    /// height is not provided, it queries the host for the current height.
    pub(super) fn determine_query_height(
        &self,
        query_height: Option<Height>,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<Height> {
        match query_height {
            Some(height) => Ok(height),
            None => {
                let ibc_ctx = IbcContext {
                    ibc: self,
                    working_set: Rc::new(RefCell::new(working_set)),
                };

                ibc_ctx.host_height().map_err(to_jsonrpsee_error)
            }
        }
    }
}

/// Creates an jsonrpsee error object
pub fn to_jsonrpsee_error(err: impl ToString) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(
        jsonrpsee::types::error::UNKNOWN_ERROR_CODE,
        err.to_string(),
        None::<String>,
    )
}

/// Trait for proof agnostic storage value retrieval.
///
/// This trait is introduced to avoid duplication in the query methods
/// that fetch values with and without proofs -
/// [`WithProof`] and [`WithoutProof`] respectively.
///
/// This trait allows to have a single query methods for both cases, e.g.:

/// let (client_state, proof) = ibc_ctx.query_client_state::<WithProof>(client_id)?;
/// let client_state = ibc_ctx.query_client_state::<WithoutProof>(client_id)?;
///
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
