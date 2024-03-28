use std::cell::RefCell;
use std::rc::Rc;

use ibc_core::client::types::Height;
use ibc_core::host::ValidationContext;
use jsonrpsee::core::RpcResult;
use jsonrpsee::types::ErrorObjectOwned;
use sov_modules_api::{Spec, WorkingSet};

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
