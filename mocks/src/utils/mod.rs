#[cfg(feature = "native")]
mod configs;

mod mutex;

use std::time::Duration;

#[cfg(feature = "native")]
pub use configs::*;
pub use mutex::*;
use tokio::time::sleep;

/// Waits for the mock chains to generate a few blocks.
pub async fn wait_for_block() {
    sleep(Duration::from_secs(1)).await;
}
