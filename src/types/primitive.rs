use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub enum Primitive {
    Boolean,
    Byte,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
}

impl TryFrom<String> for Primitive {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "boolean" => Ok(Self::Boolean),
            "byte" => Ok(Self::Byte),
            "char" => Ok(Self::Char),
            "short" => Ok(Self::Short),
            "int" => Ok(Self::Int),
            "long" => Ok(Self::Long),
            "float" => Ok(Self::Float),
            "double" => Ok(Self::Double),
            _ => Err(format!("unknown primitive: {value:?}")),
        }
    }
}
