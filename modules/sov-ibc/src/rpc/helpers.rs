use core::marker::PhantomData;
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
pub trait StorageValue<V> {
    type Output;
    fn value_at_key<K, C, S>(
        key: &K,
        store: &StateMap<K, V, C>,
        working_set: &mut WorkingSet<S>,
    ) -> Self::Output
    where
        S: Spec,
        C: StateCodec,
        <C as StateCodec>::ValueCodec: StateItemCodec<V>,
        <C as StateCodec>::KeyCodec: StateItemCodec<K>;
}

/// Implementation of [`StorageValue`] for values without proofs.
pub struct WithoutProof<V>(PhantomData<V>);

impl<V> StorageValue<V> for WithoutProof<V> {
    type Output = Option<V>;

    fn value_at_key<K, C, S>(
        key: &K,
        store: &StateMap<K, V, C>,
        working_set: &mut WorkingSet<S>,
    ) -> Self::Output
    where
        S: Spec,
        C: StateCodec,
        <C as StateCodec>::ValueCodec: StateItemCodec<V>,
        <C as StateCodec>::KeyCodec: StateItemCodec<K>,
    {
        store.get(key, working_set)
    }
}

/// Implementation of [`StorageValue`] for values with proofs.
pub struct WithProof<V>(PhantomData<V>);

impl<V> StorageValue<V> for WithProof<V> {
    type Output = (Option<V>, Vec<u8>);

    fn value_at_key<K, C, S>(
        key: &K,
        store: &StateMap<K, V, C>,
        working_set: &mut WorkingSet<S>,
    ) -> Self::Output
    where
        S: Spec,
        C: StateCodec,
        <C as StateCodec>::ValueCodec: StateItemCodec<V>,
        <C as StateCodec>::KeyCodec: StateItemCodec<K>,
    {
        let result = store.get_with_proof(key, working_set);

        (
            result
                .value
                .map(|bytes| store.codec().value_codec().decode_unwrap(bytes.value())),
            result.proof.try_to_vec().expect("no error"),
        )
    }
}
