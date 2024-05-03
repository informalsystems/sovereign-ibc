pub mod client;
pub mod transfer;

use ibc_core::primitives::proto::{Any, Protobuf};
use prost::Message;

#[test]
fn test_create_and_update_client() {
    let client_state_any = Any {
        type_url: "/ibc.lightclients.wasm.v1.ClientState".into(),
        value: [
            10, 178, 1, 10, 53, 47, 105, 98, 99, 46, 108, 105, 103, 104, 116, 99, 108, 105, 101,
            110, 116, 115, 46, 115, 111, 118, 101, 114, 101, 105, 103, 110, 46, 116, 101, 110, 100,
            101, 114, 109, 105, 110, 116, 46, 118, 49, 46, 67, 108, 105, 101, 110, 116, 83, 116,
            97, 116, 101, 18, 121, 10, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 18, 34, 10, 32, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 26, 2, 16, 10, 42, 12,
            115, 111, 118, 95, 105, 98, 99, 47, 73, 98, 99, 47, 50, 31, 10, 7, 112, 114, 105, 118,
            97, 116, 101, 18, 4, 8, 1, 16, 3, 26, 4, 8, 128, 244, 3, 34, 4, 8, 128, 232, 7, 42, 2,
            8, 3, 18, 32, 100, 49, 132, 42, 90, 61, 135, 112, 36, 34, 217, 121, 7, 134, 158, 187,
            231, 183, 33, 203, 226, 129, 201, 109, 218, 39, 104, 99, 63, 123, 190, 53, 26, 2, 16,
            10,
        ]
        .into(),
    };

    let consensus_state_any = Any {
        type_url: "/ibc.lightclients.wasm.v1.ConsensusState".into(),
        value: [
            10, 115, 10, 56, 47, 105, 98, 99, 46, 108, 105, 103, 104, 116, 99, 108, 105, 101, 110,
            116, 115, 46, 115, 111, 118, 101, 114, 101, 105, 103, 110, 46, 116, 101, 110, 100, 101,
            114, 109, 105, 110, 116, 46, 118, 49, 46, 67, 111, 110, 115, 101, 110, 115, 117, 115,
            83, 116, 97, 116, 101, 18, 55, 10, 3, 10, 1, 0, 18, 48, 10, 12, 8, 155, 241, 210, 177,
            6, 16, 239, 214, 200, 238, 1, 18, 32, 214, 185, 57, 34, 195, 58, 174, 190, 201, 4, 53,
            102, 203, 75, 27, 72, 54, 91, 19, 88, 182, 124, 125, 239, 152, 109, 158, 225, 134, 27,
            193, 67,
        ]
        .into(),
    };

    let update_client_msg_any = Any {
        type_url: "/ibc.lightclients.wasm.v1.ClientMessage".into(),
        value: [
            10, 202, 9, 10, 48, 47, 105, 98, 99, 46, 108, 105, 103, 104, 116, 99, 108, 105, 101,
            110, 116, 115, 46, 115, 111, 118, 101, 114, 101, 105, 103, 110, 46, 116, 101, 110, 100,
            101, 114, 109, 105, 110, 116, 46, 118, 49, 46, 72, 101, 97, 100, 101, 114, 18, 149, 9,
            10, 242, 6, 10, 203, 4, 10, 143, 3, 10, 4, 8, 11, 16, 1, 18, 7, 112, 114, 105, 118, 97,
            116, 101, 24, 18, 34, 12, 8, 154, 241, 210, 177, 6, 16, 167, 176, 227, 205, 2, 42, 72,
            10, 32, 18, 213, 246, 150, 78, 73, 59, 187, 26, 71, 149, 6, 82, 237, 11, 227, 160, 126,
            45, 141, 37, 118, 164, 62, 64, 87, 84, 165, 175, 59, 30, 111, 18, 36, 8, 1, 18, 32, 93,
            87, 143, 24, 115, 102, 195, 127, 244, 84, 159, 47, 85, 53, 157, 40, 80, 203, 235, 12,
            104, 124, 193, 182, 80, 9, 176, 49, 1, 127, 166, 184, 50, 32, 150, 44, 199, 63, 37, 42,
            118, 191, 230, 164, 63, 224, 248, 116, 106, 62, 144, 166, 209, 135, 111, 16, 40, 56,
            171, 71, 148, 201, 50, 14, 202, 218, 58, 32, 61, 150, 183, 210, 56, 231, 224, 69, 111,
            106, 248, 231, 205, 240, 166, 123, 214, 207, 156, 32, 137, 236, 181, 89, 198, 89, 220,
            170, 31, 136, 3, 83, 66, 32, 133, 155, 32, 133, 129, 89, 57, 28, 94, 161, 82, 121, 232,
            192, 79, 145, 39, 131, 232, 76, 104, 33, 251, 197, 35, 39, 205, 11, 1, 179, 202, 83,
            74, 32, 133, 155, 32, 133, 129, 89, 57, 28, 94, 161, 82, 121, 232, 192, 79, 145, 39,
            131, 232, 76, 104, 33, 251, 197, 35, 39, 205, 11, 1, 179, 202, 83, 82, 32, 192, 182,
            166, 52, 183, 42, 233, 104, 126, 165, 59, 109, 39, 122, 115, 171, 161, 56, 107, 163,
            207, 198, 208, 242, 105, 99, 96, 47, 127, 111, 252, 214, 90, 32, 194, 40, 98, 242, 245,
            254, 28, 139, 37, 186, 37, 66, 76, 16, 96, 82, 195, 209, 197, 79, 235, 130, 136, 161,
            238, 212, 6, 4, 61, 217, 207, 166, 98, 32, 227, 176, 196, 66, 152, 252, 28, 20, 154,
            251, 244, 200, 153, 111, 185, 36, 39, 174, 65, 228, 100, 155, 147, 76, 164, 149, 153,
            27, 120, 82, 184, 85, 106, 32, 227, 176, 196, 66, 152, 252, 28, 20, 154, 251, 244, 200,
            153, 111, 185, 36, 39, 174, 65, 228, 100, 155, 147, 76, 164, 149, 153, 27, 120, 82,
            184, 85, 114, 20, 31, 39, 125, 13, 8, 198, 108, 212, 240, 3, 135, 12, 43, 111, 36, 31,
            17, 230, 146, 229, 18, 182, 1, 8, 18, 26, 72, 10, 32, 201, 171, 178, 218, 146, 156,
            131, 211, 89, 197, 170, 207, 13, 69, 78, 244, 43, 254, 90, 177, 10, 11, 237, 48, 250,
            80, 195, 26, 80, 11, 17, 231, 18, 36, 8, 1, 18, 32, 245, 196, 143, 136, 76, 47, 246,
            98, 25, 142, 216, 237, 53, 82, 149, 61, 240, 71, 104, 75, 45, 189, 191, 172, 244, 111,
            248, 59, 214, 99, 141, 234, 34, 104, 8, 2, 18, 20, 31, 39, 125, 13, 8, 198, 108, 212,
            240, 3, 135, 12, 43, 111, 36, 31, 17, 230, 146, 229, 26, 12, 8, 155, 241, 210, 177, 6,
            16, 213, 189, 240, 236, 2, 34, 64, 18, 201, 211, 118, 144, 150, 215, 159, 14, 210, 117,
            161, 105, 78, 2, 63, 214, 93, 41, 19, 86, 205, 10, 84, 36, 195, 156, 53, 122, 46, 94,
            5, 13, 173, 92, 127, 243, 63, 94, 67, 156, 20, 198, 117, 20, 117, 242, 84, 149, 119,
            211, 13, 78, 222, 255, 225, 246, 211, 66, 87, 187, 163, 41, 6, 18, 141, 1, 10, 65, 10,
            20, 31, 39, 125, 13, 8, 198, 108, 212, 240, 3, 135, 12, 43, 111, 36, 31, 17, 230, 146,
            229, 18, 34, 10, 32, 181, 134, 28, 210, 241, 43, 191, 230, 235, 226, 189, 176, 97, 54,
            42, 168, 153, 238, 204, 220, 58, 218, 223, 37, 52, 25, 3, 33, 84, 75, 109, 163, 24,
            128, 160, 148, 165, 141, 29, 18, 65, 10, 20, 31, 39, 125, 13, 8, 198, 108, 212, 240, 3,
            135, 12, 43, 111, 36, 31, 17, 230, 146, 229, 18, 34, 10, 32, 181, 134, 28, 210, 241,
            43, 191, 230, 235, 226, 189, 176, 97, 54, 42, 168, 153, 238, 204, 220, 58, 218, 223,
            37, 52, 25, 3, 33, 84, 75, 109, 163, 24, 128, 160, 148, 165, 141, 29, 24, 128, 160,
            148, 165, 141, 29, 26, 2, 16, 10, 34, 141, 1, 10, 65, 10, 20, 31, 39, 125, 13, 8, 198,
            108, 212, 240, 3, 135, 12, 43, 111, 36, 31, 17, 230, 146, 229, 18, 34, 10, 32, 181,
            134, 28, 210, 241, 43, 191, 230, 235, 226, 189, 176, 97, 54, 42, 168, 153, 238, 204,
            220, 58, 218, 223, 37, 52, 25, 3, 33, 84, 75, 109, 163, 24, 128, 160, 148, 165, 141,
            29, 18, 65, 10, 20, 31, 39, 125, 13, 8, 198, 108, 212, 240, 3, 135, 12, 43, 111, 36,
            31, 17, 230, 146, 229, 18, 34, 10, 32, 181, 134, 28, 210, 241, 43, 191, 230, 235, 226,
            189, 176, 97, 54, 42, 168, 153, 238, 204, 220, 58, 218, 223, 37, 52, 25, 3, 33, 84, 75,
            109, 163, 24, 128, 160, 148, 165, 141, 29, 24, 128, 160, 148, 165, 141, 29, 18, 157, 2,
            10, 246, 1, 10, 34, 10, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 10, 24, 18, 34, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 42, 32, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            50, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 58, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 66, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 74, 34, 10, 32, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 18, 34,
            10, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ]
        .into(),
    };

    let client_state =
        ibc_client_wasm_types::client_message::ClientMessage::decode_vec(&client_state_any.value)
            .unwrap();

    let consensus_state = ibc_client_wasm_types::consensus_state::ConsensusState::try_from(
        consensus_state_any.clone(),
    )
    .unwrap();

    let client_msg = ibc_client_wasm_types::client_message::ClientMessage::decode_vec(
        &update_client_msg_any.value,
    )
    .unwrap();

    let sov_client_state = sov_celestia_client::types::client_state::ClientState::try_from(
        Any::decode(client_state.data.as_slice()).unwrap(),
    )
    .unwrap();

    let sov_consensus_state =
        sov_celestia_client::types::consensus_state::ConsensusState::try_from(
            Any::decode(consensus_state.data.as_slice()).unwrap(),
        )
        .unwrap();

    let sov_client_header = sov_celestia_client::types::client_message::SovTmHeader::try_from(
        Any::decode(client_msg.data.as_slice()).unwrap(),
    )
    .unwrap();

    assert_eq!(
        sov_client_state.latest_height,
        sov_client_header.da_header.trusted_height
    );

    assert_eq!(
        sov_consensus_state.da_params.next_validators_hash,
        sov_client_header
            .da_header
            .trusted_next_validator_set
            .hash()
    );
}
