use crate::{ClassId, MethodId, Type};
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(remote = "Self", deny_unknown_fields)]
pub struct Method {
    pub id: MethodId,
    pub owner: ClassId,
    pub runtime_name: String,
    #[serde(rename = "static")]
    pub is_static: bool,
    pub parameters: Vec<Type>,
    pub returns: Type,
}

object_deserialize!(Method);
