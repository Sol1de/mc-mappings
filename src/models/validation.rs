use crate::Type;
use std::{error::Error, fmt};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationError(pub String);

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for ValidationError {}

pub(super) fn require(condition: bool, message: impl Into<String>) -> Result<(), ValidationError> {
    if condition {
        Ok(())
    } else {
        Err(ValidationError(message.into()))
    }
}

pub(super) fn class_name(name: &str) -> Result<(), ValidationError> {
    require(
        !name
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, '.' | ';' | '['))
            && name.split('/').all(|part| !part.is_empty()),
        format!("invalid runtime class name: {name:?}"),
    )
}

pub(super) fn member_name(name: &str, method: bool) -> Result<(), ValidationError> {
    if method && matches!(name, "<init>" | "<clinit>") {
        return Ok(());
    }
    require(
        !name.is_empty()
            && !name
                .chars()
                .any(|c| c.is_whitespace() || matches!(c, '.' | ';' | '[' | '/' | '<' | '>')),
        format!("invalid runtime member name: {name:?}"),
    )
}

pub(super) fn validate_type(
    value: &Type,
    allow_void: bool,
    dimensions: u16,
) -> Result<(), ValidationError> {
    match value {
        Type::Void => require(allow_void, "void is only allowed as a method return"),
        Type::External(name) => class_name(name),
        Type::Array(element) => {
            require(dimensions < 255, "array exceeds 255 dimensions")?;
            validate_type(element, false, dimensions + 1)
        }
        _ => Ok(()),
    }
}
