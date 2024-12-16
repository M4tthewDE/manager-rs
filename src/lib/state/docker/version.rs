use crate::proto;

pub struct Version {
    pub version: String,
    pub api_version: String,
}

impl From<&proto::Version> for Version {
    fn from(v: &proto::Version) -> Self {
        Version {
            version: v.version.clone(),
            api_version: v.api_version.clone(),
        }
    }
}

impl Default for Version {
    fn default() -> Self {
        Self {
            version: "n/a".to_string(),
            api_version: "n/a".to_string(),
        }
    }
}
