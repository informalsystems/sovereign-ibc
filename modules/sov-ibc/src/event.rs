//! Contains event processing logic.

use ibc_core::client::types::error::ClientError;
use ibc_core::handler::types::error::ContextError;
use ibc_core::handler::types::events::IbcEvent;
use ibc_core::host::types::identifiers::{ChannelId, PortId, Sequence};
use tendermint::abci::Event as TmEvent;

/// Processes an IBC event and appends additional packet event with a hashed key
/// if the event is of `SendPacket` or `ReceivePacket` type.
pub(crate) fn process_events(event: IbcEvent) -> Result<Vec<TmEvent>, ContextError> {
    let mut events = Vec::new();

    let mut tm_event = into_tm_event(event.clone())?;

    events.push(tm_event.clone());

    match event {
        IbcEvent::SendPacket(e) => {
            tm_event.kind = compute_packet_key(
                e.port_id_on_a(),
                e.chan_id_on_a(),
                e.port_id_on_b(),
                e.chan_id_on_b(),
                e.seq_on_a(),
            );
            // This event serves as additional convenient event enabling
            // relayers to index the packet date required for processing
            // `unreceived_packets` on the target chain
            events.push(tm_event);
        }
        IbcEvent::ReceivePacket(e) => {
            tm_event.kind = compute_packet_key(
                e.port_id_on_a(),
                e.chan_id_on_a(),
                e.port_id_on_b(),
                e.chan_id_on_b(),
                e.seq_on_b(),
            );

            // This event serves as additional convenient event enabling
            // relayers to index the packet date required for processing
            // `unreceived_acknowledgements` on the target chain
            events.push(tm_event);
        }
        _ => {}
    };

    Ok(events)
}

/// Converts an IBC event into a Tendermint event.
///
/// Note: we transform IBC events into Tendermint events as an interim solution
/// to simplify the conversion process, avoiding the need for converting
/// individual IBC event types into a key-value pair of `&str`
pub(crate) fn into_tm_event(event: IbcEvent) -> Result<TmEvent, ContextError> {
    let tm_event = TmEvent::try_from(event).map_err(|e| ClientError::Other {
        description: e.to_string(),
    })?;

    Ok(tm_event)
}

/// Computes the unique base64-encoded key for either a `SendPacket` or
/// `ReceivePacket` event. This key is typically utilized to index packet data
/// required for processing pending packets by IBC relayers.
pub fn compute_packet_key(
    port_id_on_a: &PortId,
    chan_id_on_a: &ChannelId,
    port_id_on_b: &PortId,
    chan_id_on_b: &ChannelId,
    sequence: &Sequence,
) -> String {
    let mut hash_input = Vec::new();

    hash_input.extend_from_slice(port_id_on_a.as_bytes());
    hash_input.extend_from_slice(chan_id_on_a.as_bytes());
    hash_input.extend_from_slice(port_id_on_b.as_bytes());
    hash_input.extend_from_slice(chan_id_on_b.as_bytes());
    hash_input.extend_from_slice(&sequence.value().to_be_bytes());

    base64_encode(&hash(&hash_input))
}

/// Helper function to hash a byte slice using SHA256.
fn hash(data: &[u8]) -> [u8; 32] {
    use sha2::Digest;
    sha2::Sha256::digest(data).into()
}

/// Helper function to base64 encode a byte slice.
fn base64_encode(data: &[u8]) -> String {
    use base64::prelude::BASE64_STANDARD;
    use base64::Engine;
    BASE64_STANDARD.encode(data)
}

#[cfg(test)]
mod tests {
    use ibc_core::host::types::identifiers::{ChannelId, PortId, Sequence};

    use super::*;

    #[test]
    fn test_compute_packet_key() {
        let port_id_on_a = PortId::transfer();
        let chan_id_on_a = ChannelId::new(0);
        let port_id_on_b = PortId::transfer();
        let chan_id_on_b = ChannelId::new(1);
        let sequence = Sequence::from(1);

        let send_packet_key = compute_packet_key(
            &port_id_on_a,
            &chan_id_on_a,
            &port_id_on_b,
            &chan_id_on_b,
            &sequence,
        );

        let recv_packet_key = compute_packet_key(
            &port_id_on_b,
            &chan_id_on_b,
            &port_id_on_a,
            &chan_id_on_a,
            &sequence,
        );

        // Ensure that the send and recv packet keys are different
        assert_ne!(send_packet_key, recv_packet_key);

        // Snapshot check ensuring the output remains consistent
        assert_eq!(
            send_packet_key,
            "VQeG65fqa8SLEyLtdjn0nI+ktiVh6U29dsvJiIbYOZU="
        );
    }
}
