impl serde::Serialize for SovereignClientParams {
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
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.sovereign.v1.SovereignClientParams", len)?;
        if true {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("genesisStateRoot", pbjson::private::base64::encode(&self.genesis_state_root).as_str())?;
        }
        if let Some(v) = self.genesis_da_height.as_ref() {
            struct_ser.serialize_field("genesisDaHeight", v)?;
        }
        if let Some(v) = self.code_commitment.as_ref() {
            struct_ser.serialize_field("codeCommitment", v)?;
        }
        if let Some(v) = self.trusting_period.as_ref() {
            struct_ser.serialize_field("trustingPeriod", v)?;
        }
        if let Some(v) = self.frozen_height.as_ref() {
            struct_ser.serialize_field("frozenHeight", v)?;
        }
        if let Some(v) = self.latest_height.as_ref() {
            struct_ser.serialize_field("latestHeight", v)?;
        }
        if true {
            struct_ser.serialize_field("upgradePath", &self.upgrade_path)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SovereignClientParams {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "genesis_state_root",
            "genesisStateRoot",
            "genesis_da_height",
            "genesisDaHeight",
            "code_commitment",
            "codeCommitment",
            "trusting_period",
            "trustingPeriod",
            "frozen_height",
            "frozenHeight",
            "latest_height",
            "latestHeight",
            "upgrade_path",
            "upgradePath",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GenesisStateRoot,
            GenesisDaHeight,
            CodeCommitment,
            TrustingPeriod,
            FrozenHeight,
            LatestHeight,
            UpgradePath,
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
                            "genesisStateRoot" | "genesis_state_root" => Ok(GeneratedField::GenesisStateRoot),
                            "genesisDaHeight" | "genesis_da_height" => Ok(GeneratedField::GenesisDaHeight),
                            "codeCommitment" | "code_commitment" => Ok(GeneratedField::CodeCommitment),
                            "trustingPeriod" | "trusting_period" => Ok(GeneratedField::TrustingPeriod),
                            "frozenHeight" | "frozen_height" => Ok(GeneratedField::FrozenHeight),
                            "latestHeight" | "latest_height" => Ok(GeneratedField::LatestHeight),
                            "upgradePath" | "upgrade_path" => Ok(GeneratedField::UpgradePath),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SovereignClientParams;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("struct ibc.lightclients.sovereign.v1.SovereignClientParams")
            }

            fn visit_map<V>(self, mut map_: V) -> core::result::Result<SovereignClientParams, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut genesis_state_root__ = None;
                let mut genesis_da_height__ = None;
                let mut code_commitment__ = None;
                let mut trusting_period__ = None;
                let mut frozen_height__ = None;
                let mut latest_height__ = None;
                let mut upgrade_path__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::GenesisStateRoot => {
                            if genesis_state_root__.is_some() {
                                return Err(serde::de::Error::duplicate_field("genesisStateRoot"));
                            }
                            genesis_state_root__ = 
                                Some(map_.next_value::<::pbjson::private::BytesDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::GenesisDaHeight => {
                            if genesis_da_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("genesisDaHeight"));
                            }
                            genesis_da_height__ = map_.next_value()?;
                        }
                        GeneratedField::CodeCommitment => {
                            if code_commitment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("codeCommitment"));
                            }
                            code_commitment__ = map_.next_value()?;
                        }
                        GeneratedField::TrustingPeriod => {
                            if trusting_period__.is_some() {
                                return Err(serde::de::Error::duplicate_field("trustingPeriod"));
                            }
                            trusting_period__ = map_.next_value()?;
                        }
                        GeneratedField::FrozenHeight => {
                            if frozen_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("frozenHeight"));
                            }
                            frozen_height__ = map_.next_value()?;
                        }
                        GeneratedField::LatestHeight => {
                            if latest_height__.is_some() {
                                return Err(serde::de::Error::duplicate_field("latestHeight"));
                            }
                            latest_height__ = map_.next_value()?;
                        }
                        GeneratedField::UpgradePath => {
                            if upgrade_path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("upgradePath"));
                            }
                            upgrade_path__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(SovereignClientParams {
                    genesis_state_root: genesis_state_root__.unwrap_or_default(),
                    genesis_da_height: genesis_da_height__,
                    code_commitment: code_commitment__,
                    trusting_period: trusting_period__,
                    frozen_height: frozen_height__,
                    latest_height: latest_height__,
                    upgrade_path: upgrade_path__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.sovereign.v1.SovereignClientParams", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SovereignConsensusParams {
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
        let mut struct_ser = serializer.serialize_struct("ibc.lightclients.sovereign.v1.SovereignConsensusParams", len)?;
        if let Some(v) = self.root.as_ref() {
            struct_ser.serialize_field("root", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SovereignConsensusParams {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "root",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Root,
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
                            "root" => Ok(GeneratedField::Root),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SovereignConsensusParams;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("struct ibc.lightclients.sovereign.v1.SovereignConsensusParams")
            }

            fn visit_map<V>(self, mut map_: V) -> core::result::Result<SovereignConsensusParams, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut root__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Root => {
                            if root__.is_some() {
                                return Err(serde::de::Error::duplicate_field("root"));
                            }
                            root__ = map_.next_value()?;
                        }
                    }
                }
                Ok(SovereignConsensusParams {
                    root: root__,
                })
            }
        }
        deserializer.deserialize_struct("ibc.lightclients.sovereign.v1.SovereignConsensusParams", FIELDS, GeneratedVisitor)
    }
}
