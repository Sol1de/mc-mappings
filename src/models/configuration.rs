use crate::ProfileId;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(try_from = "String")]
#[non_exhaustive]
pub enum Launcher {
    Standard,
    Lunar,
}

impl Launcher {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Lunar => "lunar",
        }
    }
}

impl TryFrom<String> for Launcher {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "standard" => Ok(Self::Standard),
            "lunar" => Ok(Self::Lunar),
            _ => Err(format!("unknown launcher: {value:?}")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(try_from = "String")]
#[non_exhaustive]
pub enum Loader {
    None,
    Fabric,
    Forge,
}

impl Loader {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Fabric => "fabric",
            Self::Forge => "forge",
        }
    }
}

impl TryFrom<String> for Loader {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "none" => Ok(Self::None),
            "fabric" => Ok(Self::Fabric),
            "forge" => Ok(Self::Forge),
            _ => Err(format!("unknown loader: {value:?}")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(remote = "Self", deny_unknown_fields)]
pub struct Configuration {
    pub minecraft_version: String,
    pub launcher: Launcher,
    pub loader: Loader,
    pub profile_id: ProfileId,
}

object_deserialize!(Configuration);
