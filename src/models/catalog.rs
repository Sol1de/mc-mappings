use crate::{ClassId, FieldId, MethodId};
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(remote = "Self", deny_unknown_fields)]
pub struct Catalog {
    pub classes: Vec<ClassId>,
    pub fields: Vec<FieldId>,
    pub methods: Vec<MethodId>,
}

object_deserialize!(Catalog);
