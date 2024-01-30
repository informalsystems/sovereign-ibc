mod da_params;
mod definition;

use alloc::str::FromStr;

pub use da_params::*;
pub use definition::*;
use ibc_core::host::types::identifiers::ClientType;

pub const SOV_CELESTIA_CLIENT_TYPE: &str = "100-sov-celestia";

/// Returns the `ClientType` for the Sovereign SDK Rollups.
pub fn sov_client_type() -> ClientType {
    ClientType::from_str(SOV_CELESTIA_CLIENT_TYPE).expect("Never fails because it's valid")
}
