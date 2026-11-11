// Serde's struct derive also accepts positional arrays; this format requires objects.
macro_rules! object_deserialize {
    ($($name:ident),+ $(,)?) => {$(
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                struct Object;
                impl<'de> serde::de::Visitor<'de> for Object {
                    type Value = $name;

                    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        f.write_str("a JSON object")
                    }

                    fn visit_map<M: serde::de::MapAccess<'de>>(self, map: M) -> Result<$name, M::Error> {
                        $name::deserialize(serde::de::value::MapAccessDeserializer::new(map))
                    }
                }
                deserializer.deserialize_map(Object)
            }
        }
    )+};
}

mod catalog;
mod class;
mod configuration;
mod dataset;
mod field;
mod method;
mod profile;
mod validation;

pub use catalog::{Catalog, VersionScope};
pub use class::Class;
pub use configuration::{Configuration, Launcher, Loader};
pub use dataset::{Dataset, SCHEMA_VERSION};
pub use field::Field;
pub use method::Method;
pub use profile::Profile;
pub use validation::ValidationError;
