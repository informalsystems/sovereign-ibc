impl serde::Serialize for ClientState {
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
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.sovereign.tendermint.v1.ClientState", len)?;
        if let Some(v) = self.sovereign_params.as_ref() {
            struct_ser.serialize_field("sovereignParams", v)?;
        }
        if let Some(v) = self.tendermint_params.as_ref() {
            struct_ser.serialize_field("tendermintParams", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ClientState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "sovereign_params",
            "sovereignParams",
            "tendermint_params",
            "tendermintParams",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SovereignParams,
            TendermintParams,
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
                            "sovereignParams" | "sovereign_params" => Ok(GeneratedField::SovereignParams),
                            "tendermintParams" | "tendermint_params" => Ok(GeneratedField::TendermintParams),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ClientState;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("struct ibc.lightclients.sovereign.tendermint.v1.ClientState")
            }

            fn visit_map<V>(self, mut map_: V) -> core::result::Result<ClientState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sovereign_params__ = None;
                let mut tendermint_params__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SovereignParams => {
                            if sovereign_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sovereignParams"));
                            }
                            sovereign_params__ = map_.next_value()?;
                        }
                        GeneratedField::TendermintParams => {
                            if tendermint_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tendermintParams"));
                            }
                            tendermint_params__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ClientState {
                    sovereign_params: sovereign_params__,
                    tendermint_params: tendermint_params__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.sovereign.tendermint.v1.ClientState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ConsensusState {
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
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.sovereign.tendermint.v1.ConsensusState", len)?;
        if let Some(v) = self.sovereign_params.as_ref() {
            struct_ser.serialize_field("sovereignParams", v)?;
        }
        if let Some(v) = self.tendermint_params.as_ref() {
            struct_ser.serialize_field("tendermintParams", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ConsensusState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "sovereign_params",
            "sovereignParams",
            "tendermint_params",
            "tendermintParams",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SovereignParams,
            TendermintParams,
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
                            "sovereignParams" | "sovereign_params" => Ok(GeneratedField::SovereignParams),
                            "tendermintParams" | "tendermint_params" => Ok(GeneratedField::TendermintParams),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ConsensusState;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("struct ibc.lightclients.sovereign.tendermint.v1.ConsensusState")
            }

            fn visit_map<V>(self, mut map_: V) -> core::result::Result<ConsensusState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sovereign_params__ = None;
                let mut tendermint_params__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SovereignParams => {
                            if sovereign_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sovereignParams"));
                            }
                            sovereign_params__ = map_.next_value()?;
                        }
                        GeneratedField::TendermintParams => {
                            if tendermint_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tendermintParams"));
                            }
                            tendermint_params__ = map_.next_value()?;
                        }
                    }
                }
                Ok(ConsensusState {
                    sovereign_params: sovereign_params__,
                    tendermint_params: tendermint_params__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.sovereign.tendermint.v1.ConsensusState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Header {
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
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.sovereign.tendermint.v1.Header", len)?;
        if let Some(v) = self.tendermint_header.as_ref() {
            struct_ser.serialize_field("tendermintHeader", v)?;
        }
        if let Some(v) = self.aggregated_proof.as_ref() {
            struct_ser.serialize_field("aggregatedProof", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Header {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tendermint_header",
            "tendermintHeader",
            "aggregated_proof",
            "aggregatedProof",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TendermintHeader,
            AggregatedProof,
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
                            "tendermintHeader" | "tendermint_header" => Ok(GeneratedField::TendermintHeader),
                            "aggregatedProof" | "aggregated_proof" => Ok(GeneratedField::AggregatedProof),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Header;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("struct ibc.lightclients.sovereign.tendermint.v1.Header")
            }

            fn visit_map<V>(self, mut map_: V) -> core::result::Result<Header, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tendermint_header__ = None;
                let mut aggregated_proof__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TendermintHeader => {
                            if tendermint_header__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tendermintHeader"));
                            }
                            tendermint_header__ = map_.next_value()?;
                        }
                        GeneratedField::AggregatedProof => {
                            if aggregated_proof__.is_some() {
                                return Err(serde::de::Error::duplicate_field("aggregatedProof"));
                            }
                            aggregated_proof__ = map_.next_value()?;
                        }
                    }
                }
                Ok(Header {
                    tendermint_header: tendermint_header__,
                    aggregated_proof: aggregated_proof__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.sovereign.tendermint.v1.Header", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Misbehaviour {
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
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.sovereign.tendermint.v1.Misbehaviour", len)?;
        if true {
            struct_ser.serialize_field("clientId", &self.client_id)?;
        }
        if let Some(v) = self.header_1.as_ref() {
            struct_ser.serialize_field("header1", v)?;
        }
        if let Some(v) = self.header_2.as_ref() {
            struct_ser.serialize_field("header2", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Misbehaviour {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "client_id",
            "clientId",
            "header_1",
            "header1",
            "header_2",
            "header2",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ClientId,
            Header1,
            Header2,
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
                            "clientId" | "client_id" => Ok(GeneratedField::ClientId),
                            "header1" | "header_1" => Ok(GeneratedField::Header1),
                            "header2" | "header_2" => Ok(GeneratedField::Header2),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Misbehaviour;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("struct ibc.lightclients.sovereign.tendermint.v1.Misbehaviour")
            }

            fn visit_map<V>(self, mut map_: V) -> core::result::Result<Misbehaviour, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut client_id__ = None;
                let mut header_1__ = None;
                let mut header_2__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ClientId => {
                            if client_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientId"));
                            }
                            client_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Header1 => {
                            if header_1__.is_some() {
                                return Err(serde::de::Error::duplicate_field("header1"));
                            }
                            header_1__ = map_.next_value()?;
                        }
                        GeneratedField::Header2 => {
                            if header_2__.is_some() {
                                return Err(serde::de::Error::duplicate_field("header2"));
                            }
                            header_2__ = map_.next_value()?;
                        }
                    }
                }
                Ok(Misbehaviour {
                    client_id: client_id__.unwrap_or_default(),
                    header_1: header_1__,
                    header_2: header_2__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.sovereign.tendermint.v1.Misbehaviour", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TendermintClientParams {
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
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.sovereign.tendermint.v1.TendermintClientParams", len)?;
        if true {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        if let Some(v) = self.trust_level.as_ref() {
            struct_ser.serialize_field("trustLevel", v)?;
        }
        if let Some(v) = self.unbonding_period.as_ref() {
            struct_ser.serialize_field("unbondingPeriod", v)?;
        }
        if let Some(v) = self.max_clock_drift.as_ref() {
            struct_ser.serialize_field("maxClockDrift", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TendermintClientParams {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain_id",
            "chainId",
            "trust_level",
            "trustLevel",
            "unbonding_period",
            "unbondingPeriod",
            "max_clock_drift",
            "maxClockDrift",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainId,
            TrustLevel,
            UnbondingPeriod,
            MaxClockDrift,
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
                            "chainId" | "chain_id" => Ok(GeneratedField::ChainId),
                            "trustLevel" | "trust_level" => Ok(GeneratedField::TrustLevel),
                            "unbondingPeriod" | "unbonding_period" => Ok(GeneratedField::UnbondingPeriod),
                            "maxClockDrift" | "max_clock_drift" => Ok(GeneratedField::MaxClockDrift),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TendermintClientParams;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("struct ibc.lightclients.sovereign.tendermint.v1.TendermintClientParams")
            }

            fn visit_map<V>(self, mut map_: V) -> core::result::Result<TendermintClientParams, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_id__ = None;
                let mut trust_level__ = None;
                let mut unbonding_period__ = None;
                let mut max_clock_drift__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TrustLevel => {
                            if trust_level__.is_some() {
                                return Err(serde::de::Error::duplicate_field("trustLevel"));
                            }
                            trust_level__ = map_.next_value()?;
                        }
                        GeneratedField::UnbondingPeriod => {
                            if unbonding_period__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unbondingPeriod"));
                            }
                            unbonding_period__ = map_.next_value()?;
                        }
                        GeneratedField::MaxClockDrift => {
                            if max_clock_drift__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxClockDrift"));
                            }
                            max_clock_drift__ = map_.next_value()?;
                        }
                    }
                }
                Ok(TendermintClientParams {
                    chain_id: chain_id__.unwrap_or_default(),
                    trust_level: trust_level__,
                    unbonding_period: unbonding_period__,
                    max_clock_drift: max_clock_drift__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.sovereign.tendermint.v1.TendermintClientParams", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TendermintConsensusParams {
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
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.sovereign.tendermint.v1.TendermintConsensusParams", len)?;
        if let Some(v) = self.timestamp.as_ref() {
            struct_ser.serialize_field("timestamp", v)?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("nextValidatorsHash", pbjson::private::base64::encode(&self.next_validators_hash).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TendermintConsensusParams {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "timestamp",
            "next_validators_hash",
            "nextValidatorsHash",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Timestamp,
            NextValidatorsHash,
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
                            "timestamp" => Ok(GeneratedField::Timestamp),
                            "nextValidatorsHash" | "next_validators_hash" => Ok(GeneratedField::NextValidatorsHash),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TendermintConsensusParams;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("struct ibc.lightclients.sovereign.tendermint.v1.TendermintConsensusParams")
            }

            fn visit_map<V>(self, mut map_: V) -> core::result::Result<TendermintConsensusParams, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut timestamp__ = None;
                let mut next_validators_hash__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Timestamp => {
                            if timestamp__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestamp"));
                            }
                            timestamp__ = map_.next_value()?;
                        }
                        GeneratedField::NextValidatorsHash => {
                            if next_validators_hash__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nextValidatorsHash"));
                            }
                            next_validators_hash__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(TendermintConsensusParams {
                    timestamp: timestamp__,
                    next_validators_hash: next_validators_hash__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.sovereign.tendermint.v1.TendermintConsensusParams", FIELDS, GeneratedVisitor)
    }
}
