use core::fmt::Display;
use std::marker::PhantomData;

use ibc::core::ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment};
use ibc_proto::protobuf::{Error, Protobuf};
use prost::Message;
use sov_state::codec::{BorshCodec, StateCodec, StateValueCodec};

#[derive(Default, Clone)]
pub struct ProtobufCodec<Raw> {
    borsh_codec: BorshCodec,
    _raw: PhantomData<Raw>,
}

impl<V, Raw> StateValueCodec<V> for ProtobufCodec<Raw>
where
    V: Protobuf<Raw>,
    V::Error: Display,
    Raw: Message + Default,
{
    type Error = Error;

    fn encode_value(&self, value: &V) -> Vec<u8> {
        value.encode_vec()
    }

    fn try_decode_value(&self, bytes: &[u8]) -> Result<V, Self::Error> {
        Protobuf::decode_vec(bytes)
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

impl StateValueCodec<PacketCommitment> for PacketCommitmentCodec {
    type Error = Error;

    fn encode_value(&self, commitment: &PacketCommitment) -> Vec<u8> {
        commitment.clone().into_vec()
    }

    fn try_decode_value(&self, bytes: &[u8]) -> Result<PacketCommitment, Self::Error> {
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

impl StateValueCodec<AcknowledgementCommitment> for AcknowledgementCommitmentCodec {
    type Error = Error;

    fn encode_value(&self, commitment: &AcknowledgementCommitment) -> Vec<u8> {
        commitment.clone().into_vec()
    }

    fn try_decode_value(&self, bytes: &[u8]) -> Result<AcknowledgementCommitment, Self::Error> {
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
