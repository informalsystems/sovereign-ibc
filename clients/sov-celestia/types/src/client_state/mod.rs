mod definition;
mod rollup;

use alloc::str::FromStr;

pub use definition::*;
use ibc_core::host::types::identifiers::ClientType;
pub use rollup::*;

pub const SOVEREIGN_CLIENT_TYPE: &str = "100-sovereign";

/// Returns the `ClientType` for the Sovereign SDK Rollups.
pub fn sov_client_type() -> ClientType {
    ClientType::from_str(SOVEREIGN_CLIENT_TYPE).expect("Never fails because it's valid")
}
