use crate::ProfileId;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(try_from = "String")]
pub enum Launcher {
    Standard,
    Lunar,
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
pub enum Loader {
    None,
    Fabric,
    Forge,
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
