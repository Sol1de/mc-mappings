use super::{Class, Field, Method};
use crate::ProfileId;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(remote = "Self", deny_unknown_fields)]
pub struct Profile {
    pub profile_id: ProfileId,
    pub classes: Vec<Class>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
}

object_deserialize!(Profile);
