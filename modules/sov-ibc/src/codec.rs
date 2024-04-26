use core::fmt::Display;
use std::io::Error;
use std::marker::PhantomData;

use ibc_core::channel::types::commitment::{AcknowledgementCommitment, PacketCommitment};
use ibc_core::primitives::proto::Protobuf;
use prost::Message;
use sov_state::codec::BorshCodec;
use sov_state::storage::{StateCodec, StateItemCodec, StateItemDecoder, StateItemEncoder};

#[derive(Default, Clone)]
pub struct ProtobufCodec<Raw> {
    borsh_codec: BorshCodec,
    _raw: PhantomData<Raw>,
}

impl<V, Raw> StateItemEncoder<V> for ProtobufCodec<Raw>
where
    V: Protobuf<Raw>,
    V::Error: Display,
    Raw: From<V> + Message + Default,
{
    fn encode(&self, value: &V) -> Vec<u8> {
        value.clone().encode_vec()
    }
}

impl<V, Raw> StateItemDecoder<V> for ProtobufCodec<Raw>
where
    V: Protobuf<Raw>,
    V::Error: Display,
    Raw: From<V> + Message + Default,
{
    type Error = Error;

    fn try_decode(&self, bytes: &[u8]) -> Result<V, Self::Error> {
        Protobuf::decode_vec(bytes).map_err(|e| {
            Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Protobuf decode error: {e}"),
            )
        })
    }
}

impl<Raw> StateCodec for ProtobufCodec<Raw> {
    type KeyCodec = BorshCodec;

    type ValueCodec = Self;

    fn key_codec(&self) -> &Self::KeyCodec {
        &self.borsh_codec
    }

    fn value_codec(&self) -> &Self::ValueCodec {
        self
    }
}

#[derive(Default, Clone)]
pub struct PacketCommitmentCodec {
    borsh_codec: BorshCodec,
}

impl StateItemEncoder<PacketCommitment> for PacketCommitmentCodec {
    fn encode(&self, commitment: &PacketCommitment) -> Vec<u8> {
        commitment.clone().into_vec()
    }
}

impl StateItemDecoder<PacketCommitment> for PacketCommitmentCodec {
    type Error = Error;

    fn try_decode(&self, bytes: &[u8]) -> Result<PacketCommitment, Self::Error> {
        Ok(PacketCommitment::from(bytes.to_vec()))
    }
}

impl StateCodec for PacketCommitmentCodec {
    type KeyCodec = BorshCodec;

    type ValueCodec = Self;

    fn key_codec(&self) -> &Self::KeyCodec {
        &self.borsh_codec
    }

    fn value_codec(&self) -> &Self::ValueCodec {
        self
    }
}

#[derive(Default, Clone)]
pub struct AcknowledgementCommitmentCodec {
    borsh_codec: BorshCodec,
}

impl StateItemEncoder<AcknowledgementCommitment> for AcknowledgementCommitmentCodec {
    fn encode(&self, commitment: &AcknowledgementCommitment) -> Vec<u8> {
        commitment.clone().into_vec()
    }
}

impl StateItemDecoder<AcknowledgementCommitment> for AcknowledgementCommitmentCodec {
    type Error = Error;

    fn try_decode(&self, bytes: &[u8]) -> Result<AcknowledgementCommitment, Self::Error> {
        Ok(AcknowledgementCommitment::from(bytes.to_vec()))
    }
}

impl StateCodec for AcknowledgementCommitmentCodec {
    type KeyCodec = BorshCodec;

    type ValueCodec = Self;

    fn key_codec(&self) -> &Self::KeyCodec {
        &self.borsh_codec
    }

    fn value_codec(&self) -> &Self::ValueCodec {
        self
    }
}
