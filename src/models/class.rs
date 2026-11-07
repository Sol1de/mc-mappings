use crate::ClassId;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(remote = "Self", deny_unknown_fields)]
pub struct Class {
    pub id: ClassId,
    pub runtime_name: String,
}

object_deserialize!(Class);
