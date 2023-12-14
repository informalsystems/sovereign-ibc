use std::time::Duration;

use tokio::time::sleep;

/// Waits for the mock chains to generate a few blocks.
pub async fn wait_for_block() {
    sleep(Duration::from_secs(1)).await;
}
