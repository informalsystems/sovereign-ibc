impl serde::Serialize for AggregatedProof {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("sovereign.types.v1.AggregatedProof", len)?;
        if let Some(v) = self.public_data.as_ref() {
            struct_ser.serialize_field("publicData", v)?;
        }
        if let Some(v) = self.serialized_proof.as_ref() {
            struct_ser.serialize_field("serializedProof", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AggregatedProof {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "public_data",
            "publicData",
            "serialized_proof",
            "serializedProof",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PublicData,
            SerializedProof,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> core::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> core::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "publicData" | "public_data" => Ok(GeneratedField::PublicData),
                            "serializedProof" | "serialized_proof" => Ok(GeneratedField::SerializedProof),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AggregatedProof;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("struct sovereign.types.v1.AggregatedProof")
            }

            fn visit_map<V>(self, mut map_: V) -> core::result::Result<AggregatedProof, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut public_data__ = None;
                let mut serialized_proof__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PublicData => {
                            if public_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("publicData"));
                            }
                            public_data__ = map_.next_value()?;
                        }
                        GeneratedField::SerializedProof => {
                            if serialized_proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("serializedProof"));
                            }
                            serialized_proof__ = map_.next_value()?;
                        }
                    }
                }
                Ok(AggregatedProof {
                    public_data: public_data__,
                    serialized_proof: serialized_proof__,
                })
            }
        }
        deserializer.deserialize_struct("sovereign.types.v1.AggregatedProof", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AggregatedProofPublicData {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("sovereign.types.v1.AggregatedProofPublicData", len)?;
        if true {
            struct_ser.serialize_field("validityConditions", &self.validity_conditions)?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("initialSlotNumber", ::alloc::string::ToString::to_string(&self.initial_slot_number).as_str())?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("finalSlotNumber", ::alloc::string::ToString::to_string(&self.final_slot_number).as_str())?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("genesisStateRoot", pbjson::private::base64::encode(&self.genesis_state_root).as_str())?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("initialStateRoot", pbjson::private::base64::encode(&self.initial_state_root).as_str())?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("finalStateRoot", pbjson::private::base64::encode(&self.final_state_root).as_str())?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("initialSlotHash", pbjson::private::base64::encode(&self.initial_slot_hash).as_str())?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("finalSlotHash", pbjson::private::base64::encode(&self.final_slot_hash).as_str())?;
        }
        if let Some(v) = self.code_commitment.as_ref() {
            struct_ser.serialize_field("codeCommitment", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AggregatedProofPublicData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validity_conditions",
            "validityConditions",
            "initial_slot_number",
            "initialSlotNumber",
            "final_slot_number",
            "finalSlotNumber",
            "genesis_state_root",
            "genesisStateRoot",
            "initial_state_root",
            "initialStateRoot",
            "final_state_root",
            "finalStateRoot",
            "initial_slot_hash",
            "initialSlotHash",
            "final_slot_hash",
            "finalSlotHash",
            "code_commitment",
            "codeCommitment",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidityConditions,
            InitialSlotNumber,
            FinalSlotNumber,
            GenesisStateRoot,
            InitialStateRoot,
            FinalStateRoot,
            InitialSlotHash,
            FinalSlotHash,
            CodeCommitment,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> core::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> core::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "validityConditions" | "validity_conditions" => Ok(GeneratedField::ValidityConditions),
                            "initialSlotNumber" | "initial_slot_number" => Ok(GeneratedField::InitialSlotNumber),
                            "finalSlotNumber" | "final_slot_number" => Ok(GeneratedField::FinalSlotNumber),
                            "genesisStateRoot" | "genesis_state_root" => Ok(GeneratedField::GenesisStateRoot),
                            "initialStateRoot" | "initial_state_root" => Ok(GeneratedField::InitialStateRoot),
                            "finalStateRoot" | "final_state_root" => Ok(GeneratedField::FinalStateRoot),
                            "initialSlotHash" | "initial_slot_hash" => Ok(GeneratedField::InitialSlotHash),
                            "finalSlotHash" | "final_slot_hash" => Ok(GeneratedField::FinalSlotHash),
                            "codeCommitment" | "code_commitment" => Ok(GeneratedField::CodeCommitment),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AggregatedProofPublicData;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("struct sovereign.types.v1.AggregatedProofPublicData")
            }

            fn visit_map<V>(self, mut map_: V) -> core::result::Result<AggregatedProofPublicData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validity_conditions__ = None;
                let mut initial_slot_number__ = None;
                let mut final_slot_number__ = None;
                let mut genesis_state_root__ = None;
                let mut initial_state_root__ = None;
                let mut final_state_root__ = None;
                let mut initial_slot_hash__ = None;
                let mut final_slot_hash__ = None;
                let mut code_commitment__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ValidityConditions => {
                            if validity_conditions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validityConditions"));
                            }
                            validity_conditions__ = Some(map_.next_value()?);
                        }
                        GeneratedField::InitialSlotNumber => {
                            if initial_slot_number__.is_some() {
                                return Err(serde::de::Error::duplicate_field("initialSlotNumber"));
                            }
                            initial_slot_number__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::FinalSlotNumber => {
                            if final_slot_number__.is_some() {
                                return Err(serde::de::Error::duplicate_field("finalSlotNumber"));
                            }
                            final_slot_number__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::GenesisStateRoot => {
                            if genesis_state_root__.is_some() {
                                return Err(serde::de::Error::duplicate_field("genesisStateRoot"));
                            }
                            genesis_state_root__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::InitialStateRoot => {
                            if initial_state_root__.is_some() {
                                return Err(serde::de::Error::duplicate_field("initialStateRoot"));
                            }
                            initial_state_root__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::FinalStateRoot => {
                            if final_state_root__.is_some() {
                                return Err(serde::de::Error::duplicate_field("finalStateRoot"));
                            }
                            final_state_root__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::InitialSlotHash => {
                            if initial_slot_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("initialSlotHash"));
                            }
                            initial_slot_hash__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::FinalSlotHash => {
                            if final_slot_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("finalSlotHash"));
                            }
                            final_slot_hash__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::CodeCommitment => {
                            if code_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("codeCommitment"));
                            }
                            code_commitment__ = map_.next_value()?;
                        }
                    }
                }
                Ok(AggregatedProofPublicData {
                    validity_conditions: validity_conditions__.unwrap_or_default(),
                    initial_slot_number: initial_slot_number__.unwrap_or_default(),
                    final_slot_number: final_slot_number__.unwrap_or_default(),
                    genesis_state_root: genesis_state_root__.unwrap_or_default(),
                    initial_state_root: initial_state_root__.unwrap_or_default(),
                    final_state_root: final_state_root__.unwrap_or_default(),
                    initial_slot_hash: initial_slot_hash__.unwrap_or_default(),
                    final_slot_hash: final_slot_hash__.unwrap_or_default(),
                    code_commitment: code_commitment__,
                })
            }
        }
        deserializer.deserialize_struct("sovereign.types.v1.AggregatedProofPublicData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CodeCommitment {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("sovereign.types.v1.CodeCommitment", len)?;
        if true {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("codeCommitment", pbjson::private::base64::encode(&self.code_commitment).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CodeCommitment {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "code_commitment",
            "codeCommitment",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CodeCommitment,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> core::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> core::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "codeCommitment" | "code_commitment" => Ok(GeneratedField::CodeCommitment),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CodeCommitment;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("struct sovereign.types.v1.CodeCommitment")
            }

            fn visit_map<V>(self, mut map_: V) -> core::result::Result<CodeCommitment, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut code_commitment__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CodeCommitment => {
                            if code_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("codeCommitment"));
                            }
                            code_commitment__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(CodeCommitment {
                    code_commitment: code_commitment__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("sovereign.types.v1.CodeCommitment", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SerializedAggregatedProof {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("sovereign.types.v1.SerializedAggregatedProof", len)?;
        if true {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("rawAggregatedProof", pbjson::private::base64::encode(&self.raw_aggregated_proof).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SerializedAggregatedProof {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "raw_aggregated_proof",
            "rawAggregatedProof",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RawAggregatedProof,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> core::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> core::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "rawAggregatedProof" | "raw_aggregated_proof" => Ok(GeneratedField::RawAggregatedProof),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SerializedAggregatedProof;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("struct sovereign.types.v1.SerializedAggregatedProof")
            }

            fn visit_map<V>(self, mut map_: V) -> core::result::Result<SerializedAggregatedProof, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut raw_aggregated_proof__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::RawAggregatedProof => {
                            if raw_aggregated_proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rawAggregatedProof"));
                            }
                            raw_aggregated_proof__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(SerializedAggregatedProof {
                    raw_aggregated_proof: raw_aggregated_proof__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("sovereign.types.v1.SerializedAggregatedProof", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SerializedValidityCondition {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("sovereign.types.v1.SerializedValidityCondition", len)?;
        if true {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("validityCondition", pbjson::private::base64::encode(&self.validity_condition).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SerializedValidityCondition {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "validity_condition",
            "validityCondition",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ValidityCondition,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> core::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> core::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "validityCondition" | "validity_condition" => Ok(GeneratedField::ValidityCondition),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SerializedValidityCondition;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("struct sovereign.types.v1.SerializedValidityCondition")
            }

            fn visit_map<V>(self, mut map_: V) -> core::result::Result<SerializedValidityCondition, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut validity_condition__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ValidityCondition => {
                            if validity_condition__.is_some() {
                                return Err(serde::de::Error::duplicate_field("validityCondition"));
                            }
                            validity_condition__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(SerializedValidityCondition {
                    validity_condition: validity_condition__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("sovereign.types.v1.SerializedValidityCondition", FIELDS, GeneratedVisitor)
    }
}
