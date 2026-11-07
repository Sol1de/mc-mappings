use serde::Deserialize;
use std::{error::Error, fmt, str::FromStr};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvalidIdentifier(pub String);

impl fmt::Display for InvalidIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid semantic identifier: {:?}", self.0)
    }
}

impl Error for InvalidIdentifier {}

fn valid_id(value: &str) -> bool {
    value.split('.').all(|segment| {
        let mut bytes = segment.bytes();
        bytes.next().is_some_and(|b| b.is_ascii_lowercase())
            && bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
    })
}

macro_rules! identifier {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
        #[serde(try_from = "String")]
        pub struct $name(String);

        impl $name {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = InvalidIdentifier;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                if valid_id(&value) {
                    Ok(Self(value))
                } else {
                    Err(InvalidIdentifier(value))
                }
            }
        }

        impl FromStr for $name {
            type Err = InvalidIdentifier;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                value.to_owned().try_into()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

identifier!(ClassId);
identifier!(FieldId);
identifier!(MethodId);
identifier!(ProfileId);
