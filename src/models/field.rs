use crate::{ClassId, FieldId, Type};
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(remote = "Self", deny_unknown_fields)]
pub struct Field {
    pub id: FieldId,
    pub owner: ClassId,
    pub runtime_name: String,
    #[serde(rename = "static")]
    pub is_static: bool,
    #[serde(rename = "type")]
    pub field_type: Type,
}

object_deserialize!(Field);
