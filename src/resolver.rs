//! Configuration selection and symbol resolution ([spec/model.md](../spec/model.md)).
//!
//! Pure data: no JVM, JNI or classloader. A selection never falls back to another version,
//! launcher or loader, and a missing symbol is an error, never a guess.

use crate::{
    ClassId, Configuration, Dataset, FieldId, Launcher, Loader, MethodId, Primitive, Profile, Type,
    normalize_minecraft_version,
};
use std::{error::Error, fmt};

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResolveError {
    /// No registry entry for this exact configuration: unlisted version (stable or not),
    /// snapshot, pre-release, or missing launcher/loader combination.
    UnsupportedConfiguration {
        minecraft_version: String,
        launcher: Launcher,
        loader: Loader,
    },
    /// The identifier is not declared by the catalog.
    UnknownSymbol(String),
    /// The catalog declares the symbol, but it does not exist in this Minecraft version.
    AbsentInVersion {
        symbol: String,
        minecraft_version: String,
    },
    /// The data is inconsistent: duplicate registry entry, missing profile, symbol, owner or
    /// mapped type reference. Run [`Dataset::audit`] for every problem at once.
    InvalidData(String),
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedConfiguration {
                minecraft_version,
                launcher,
                loader,
            } => write!(
                f,
                "unsupported configuration: Minecraft {minecraft_version:?}, launcher {}, loader {}",
                launcher.as_str(),
                loader.as_str()
            ),
            Self::UnknownSymbol(id) => write!(f, "unknown symbol {id}"),
            Self::AbsentInVersion {
                symbol,
                minecraft_version,
            } => write!(
                f,
                "{symbol} does not exist in Minecraft {minecraft_version}"
            ),
            Self::InvalidData(message) => write!(f, "invalid data: {message}"),
        }
    }
}

impl Error for ResolveError {}

fn invalid(message: impl Into<String>) -> ResolveError {
    ResolveError::InvalidData(message.into())
}

/// The profile selected for one configuration.
#[derive(Clone, Copy, Debug)]
pub struct Selection<'a> {
    pub dataset: &'a Dataset,
    pub configuration: &'a Configuration,
    pub profile: &'a Profile,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedClass<'a> {
    pub id: &'a ClassId,
    /// Slash-separated internal name, e.g. `net/minecraft/class_310`.
    pub runtime_name: &'a str,
}

/// A field or method resolved to its runtime declaring class, name and JVM descriptor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedMember<'a> {
    /// Runtime internal name of the declaring class.
    pub owner: &'a str,
    pub runtime_name: &'a str,
    /// Complete JVM descriptor with runtime class names (`I`, `(ILjava/lang/String;)[La;`).
    pub descriptor: String,
    pub is_static: bool,
}

impl Dataset {
    /// Selects the profile of the exact `(minecraft_version, launcher, loader)` configuration.
    /// `1.21.0` is the only alias (of `1.21`); every other string is matched verbatim.
    pub fn select(
        &self,
        minecraft_version: &str,
        launcher: Launcher,
        loader: Loader,
    ) -> Result<Selection<'_>, ResolveError> {
        let version = normalize_minecraft_version(minecraft_version);
        let mut matches = self.configurations.iter().filter(|c| {
            c.minecraft_version == version && c.launcher == launcher && c.loader == loader
        });
        let Some(configuration) = matches.next() else {
            return Err(ResolveError::UnsupportedConfiguration {
                minecraft_version: minecraft_version.to_owned(),
                launcher,
                loader,
            });
        };
        if matches.next().is_some() {
            return Err(invalid(format!(
                "several registry entries for {version} {} {}",
                launcher.as_str(),
                loader.as_str()
            )));
        }
        let mut profiles = self
            .profiles
            .iter()
            .filter(|p| p.profile_id == configuration.profile_id);
        let profile = profiles
            .next()
            .ok_or_else(|| invalid(format!("missing profile {}", configuration.profile_id)))?;
        if profiles.next().is_some() {
            return Err(invalid(format!(
                "duplicate profile {}",
                configuration.profile_id
            )));
        }
        Ok(Selection {
            dataset: self,
            configuration,
            profile,
        })
    }
}

fn primitive(p: Primitive) -> char {
    match p {
        Primitive::Boolean => 'Z',
        Primitive::Byte => 'B',
        Primitive::Char => 'C',
        Primitive::Short => 'S',
        Primitive::Int => 'I',
        Primitive::Long => 'J',
        Primitive::Float => 'F',
        Primitive::Double => 'D',
    }
}

impl<'a> Selection<'a> {
    fn profile_id(&self) -> &str {
        self.profile.profile_id.as_str()
    }

    /// Fails for a catalog symbol scoped to other Minecraft versions than the selected one.
    fn present(&self, id: &str) -> Result<(), ResolveError> {
        let version = &self.configuration.minecraft_version;
        if self.dataset.catalog.exists_in(id, version) {
            Ok(())
        } else {
            Err(ResolveError::AbsentInVersion {
                symbol: id.to_owned(),
                minecraft_version: version.clone(),
            })
        }
    }

    pub fn class(&self, id: &ClassId) -> Result<ResolvedClass<'a>, ResolveError> {
        if !self.dataset.catalog.classes.contains(id) {
            return Err(ResolveError::UnknownSymbol(id.to_string()));
        }
        self.present(id.as_str())?;
        let record = self
            .profile
            .classes
            .iter()
            .find(|c| &c.id == id)
            .ok_or_else(|| invalid(format!("profile {} lacks class {id}", self.profile_id())))?;
        Ok(ResolvedClass {
            id: &record.id,
            runtime_name: &record.runtime_name,
        })
    }

    /// Appends the JVM descriptor of `ty`, translating mapped classes into this profile's names.
    fn push_descriptor(&self, ty: &Type, out: &mut String) -> Result<(), ResolveError> {
        match ty {
            Type::Primitive(p) => out.push(primitive(*p)),
            Type::Void => out.push('V'),
            Type::Class(id) => {
                out.push('L');
                out.push_str(
                    self.class(id)
                        .map_err(|e| match e {
                            ResolveError::UnknownSymbol(_) => {
                                invalid(format!("type references undeclared class {id}"))
                            }
                            ResolveError::AbsentInVersion { .. } => invalid(format!(
                                "type references class {id}, absent in this Minecraft version"
                            )),
                            other => other,
                        })?
                        .runtime_name,
                );
                out.push(';');
            }
            Type::External(name) => {
                out.push('L');
                out.push_str(name);
                out.push(';');
            }
            Type::Array(element) => {
                out.push('[');
                self.push_descriptor(element, out)?;
            }
        }
        Ok(())
    }

    /// JVM descriptor of a type in this profile (for a field type, a parameter or a return).
    pub fn descriptor(&self, ty: &Type) -> Result<String, ResolveError> {
        let mut out = String::new();
        self.push_descriptor(ty, &mut out)?;
        Ok(out)
    }

    fn owner(&self, owner: &ClassId, member: &str) -> Result<&'a str, ResolveError> {
        self.class(owner).map(|c| c.runtime_name).map_err(|_| {
            invalid(format!(
                "profile {}: owner {owner} of {member} is missing",
                self.profile_id()
            ))
        })
    }

    pub fn field(&self, id: &FieldId) -> Result<ResolvedMember<'a>, ResolveError> {
        if !self.dataset.catalog.fields.contains(id) {
            return Err(ResolveError::UnknownSymbol(id.to_string()));
        }
        self.present(id.as_str())?;
        let record = self
            .profile
            .fields
            .iter()
            .find(|f| &f.id == id)
            .ok_or_else(|| invalid(format!("profile {} lacks field {id}", self.profile_id())))?;
        Ok(ResolvedMember {
            owner: self.owner(&record.owner, id.as_str())?,
            runtime_name: &record.runtime_name,
            descriptor: self.descriptor(&record.field_type)?,
            is_static: record.is_static,
        })
    }

    /// Each overload has its own identifier; there is no lookup by bare runtime name.
    pub fn method(&self, id: &MethodId) -> Result<ResolvedMember<'a>, ResolveError> {
        if !self.dataset.catalog.methods.contains(id) {
            return Err(ResolveError::UnknownSymbol(id.to_string()));
        }
        self.present(id.as_str())?;
        let record = self
            .profile
            .methods
            .iter()
            .find(|m| &m.id == id)
            .ok_or_else(|| invalid(format!("profile {} lacks method {id}", self.profile_id())))?;
        let mut descriptor = String::from("(");
        for parameter in &record.parameters {
            self.push_descriptor(parameter, &mut descriptor)?;
        }
        descriptor.push(')');
        self.push_descriptor(&record.returns, &mut descriptor)?;
        Ok(ResolvedMember {
            owner: self.owner(&record.owner, id.as_str())?,
            runtime_name: &record.runtime_name,
            descriptor,
            is_static: record.is_static,
        })
    }
}
