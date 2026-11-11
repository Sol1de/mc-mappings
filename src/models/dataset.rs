use super::validation::{class_name, member_name, require, validate_type};
use super::{Catalog, Configuration, Launcher, Loader, Profile, ValidationError};
use crate::Type;
use serde::Deserialize;
use std::num::NonZeroU64;

/// The current schema. Schema 1 (no version-scoped symbols) is still read.
pub const SCHEMA_VERSION: u64 = 2;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "RawDataset")]
pub struct Dataset {
    pub schema_version: u64,
    pub data_version: NonZeroU64,
    pub catalog: Catalog,
    pub configurations: Vec<Configuration>,
    pub profiles: Vec<Profile>,
}

#[derive(Deserialize)]
#[serde(remote = "Self", deny_unknown_fields)]
struct RawDataset {
    schema_version: u64,
    data_version: NonZeroU64,
    catalog: Catalog,
    configurations: Vec<Configuration>,
    profiles: Vec<Profile>,
}

impl TryFrom<RawDataset> for Dataset {
    type Error = ValidationError;

    fn try_from(raw: RawDataset) -> Result<Self, Self::Error> {
        let dataset = Self {
            schema_version: raw.schema_version,
            data_version: raw.data_version,
            catalog: raw.catalog,
            configurations: raw.configurations,
            profiles: raw.profiles,
        };
        dataset.validate()?;
        Ok(dataset)
    }
}

object_deserialize!(RawDataset);

impl Dataset {
    /// Parse strict JSON and validate local constraints, without auditing references.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let mut deserializer = serde_json::Deserializer::from_str(json);
        // TypeSeed bounds recursive types; the default JSON limit cannot fit 255 arrays.
        deserializer.disable_recursion_limit();
        let dataset = Self::deserialize(&mut deserializer)?;
        deserializer.end()?;
        Ok(dataset)
    }

    /// Recheck local constraints after constructing or modifying data in Rust.
    /// Does not check cross-table references, uniqueness or catalog coverage.
    pub fn validate(&self) -> Result<(), ValidationError> {
        require(
            matches!(self.schema_version, 1 | SCHEMA_VERSION),
            "unsupported schema_version",
        )?;
        require(
            self.schema_version > 1 || self.catalog.scoped.is_empty(),
            "version-scoped symbols need schema_version 2",
        )?;
        let in_scope = |version: &String| {
            require(
                matches!(version.as_str(), "1.21" | "1.21.11"),
                "stored minecraft_version must be canonical and within the approved scope",
            )
        };
        for scope in &self.catalog.scoped {
            require(
                !scope.minecraft_versions.is_empty(),
                "a version scope must list at least one Minecraft version",
            )?;
            scope.minecraft_versions.iter().try_for_each(in_scope)?;
        }
        for configuration in &self.configurations {
            in_scope(&configuration.minecraft_version)?;
            require(
                !(configuration.launcher == Launcher::Lunar
                    && configuration.loader == Loader::Forge),
                "Lunar/Forge is outside the approved configurations",
            )?;
        }
        for profile in &self.profiles {
            for class in &profile.classes {
                class_name(&class.runtime_name)?;
            }
            for field in &profile.fields {
                member_name(&field.runtime_name, false)?;
                validate_type(&field.field_type, false, 0)?;
            }
            for method in &profile.methods {
                member_name(&method.runtime_name, true)?;
                for parameter in &method.parameters {
                    validate_type(parameter, false, 0)?;
                }
                validate_type(&method.returns, true, 0)?;
                match method.runtime_name.as_str() {
                    "<init>" => require(
                        !method.is_static && method.returns == Type::Void,
                        "constructor must be non-static and return void",
                    )?,
                    "<clinit>" => require(
                        method.is_static
                            && method.parameters.is_empty()
                            && method.returns == Type::Void,
                        "class initializer must be static, parameterless and return void",
                    )?,
                    _ => (),
                }
            }
        }
        Ok(())
    }
}
