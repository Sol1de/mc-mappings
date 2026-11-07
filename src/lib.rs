//! Mapping table data, without a JVM or symbol resolver.
//!
//! [`Dataset::from_json`] checks structure and local constraints. Cross-table
//! references, uniqueness and catalog coverage require a separate audit (MM-7).

mod identifiers;
mod models;
mod types;

pub use identifiers::{ClassId, FieldId, InvalidIdentifier, MethodId, ProfileId};
pub use models::*;
pub use types::{Primitive, Type};

/// The only accepted input alias. Other strings remain exact identities.
pub fn normalize_minecraft_version(version: &str) -> &str {
    match version {
        "1.21.0" => "1.21",
        other => other,
    }
}
