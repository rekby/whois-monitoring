use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub(crate) struct CustomerConfig {
    pub name: String,

    #[serde(default)]
    pub disabled: bool,
    pub emails: Vec<String>,
    pub domains: Vec<DomainConfig>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub(crate) struct DomainConfig {
    pub domain: String,

    #[serde(default)]
    pub account: String,

    #[serde(default)]
    pub autorenew: bool,

    #[serde(default)]
    pub disabled: bool,
}
