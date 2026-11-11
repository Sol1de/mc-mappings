use crate::{ClassId, FieldId, MethodId};
use serde::Deserialize;

/// A catalog symbol that exists only in the listed Minecraft versions (schema 2).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(remote = "Self", deny_unknown_fields)]
pub struct VersionScope {
    /// Identifier of a catalog class, field or method.
    pub id: String,
    pub minecraft_versions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(remote = "Self", deny_unknown_fields)]
pub struct Catalog {
    pub classes: Vec<ClassId>,
    pub fields: Vec<FieldId>,
    pub methods: Vec<MethodId>,
    /// Symbols limited to some Minecraft versions; every other symbol exists in all of them.
    #[serde(default)]
    pub scoped: Vec<VersionScope>,
}

impl Catalog {
    /// Whether the catalog symbol `id` exists in `minecraft_version` (canonical spelling).
    pub fn exists_in(&self, id: &str, minecraft_version: &str) -> bool {
        self.scoped
            .iter()
            .filter(|s| s.id == id)
            .all(|s| s.minecraft_versions.iter().any(|v| v == minecraft_version))
    }
}

object_deserialize!(VersionScope, Catalog);
