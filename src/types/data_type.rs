use super::Primitive;
use crate::ClassId;
use serde::{
    Deserialize, Deserializer,
    de::{self, DeserializeSeed, MapAccess, Visitor},
};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Primitive(Primitive),
    Class(ClassId),
    External(String),
    Array(Box<Type>),
    Void,
}

impl<'de> Deserialize<'de> for Type {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        TypeSeed(0).deserialize(deserializer)
    }
}

struct TypeSeed(u16);

impl<'de> DeserializeSeed<'de> for TypeSeed {
    type Value = Type;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Type, D::Error> {
        if self.0 > 255 {
            return Err(de::Error::custom("array exceeds 255 dimensions"));
        }
        deserializer.deserialize_map(self)
    }
}

impl<'de> Visitor<'de> for TypeSeed {
    type Value = Type;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a structured type object")
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Type, M::Error> {
        let (mut kind, mut name, mut id, mut runtime_name, mut element) =
            (None::<String>, None, None, None, None);
        let mut seen = 0u8;
        while let Some(key) = map.next_key::<String>()? {
            let bit = match key.as_str() {
                "kind" => 1,
                "name" => 2,
                "id" => 4,
                "runtime_name" => 8,
                "element" => 16,
                _ => {
                    return Err(de::Error::unknown_field(
                        &key,
                        &["kind", "name", "id", "runtime_name", "element"],
                    ));
                }
            };
            if seen & bit != 0 {
                return Err(de::Error::custom(format!("duplicate type field: {key}")));
            }
            seen |= bit;
            match key.as_str() {
                "kind" => kind = Some(map.next_value()?),
                "name" => name = Some(map.next_value::<Primitive>()?),
                "id" => id = Some(map.next_value::<ClassId>()?),
                "runtime_name" => runtime_name = Some(map.next_value::<String>()?),
                "element" => element = Some(map.next_value_seed(TypeSeed(self.0 + 1))?),
                _ => unreachable!(),
            }
        }
        let kind = kind.ok_or_else(|| de::Error::missing_field("kind"))?;
        match (kind.as_str(), seen) {
            ("primitive", 3) => Ok(Type::Primitive(name.unwrap())),
            ("class", 5) => Ok(Type::Class(id.unwrap())),
            ("external", 9) => Ok(Type::External(runtime_name.unwrap())),
            ("array", 17) => Ok(Type::Array(Box::new(element.unwrap()))),
            ("void", 1) => Ok(Type::Void),
            _ => Err(de::Error::custom(format!(
                "invalid fields or kind for type {kind:?}"
            ))),
        }
    }
}
