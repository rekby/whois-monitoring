use crate::errors::Error;
use serde_derive::Deserialize;

type Result<T> = std::result::Result<T, Error>;

pub(super) const DEFAULT_CONFIG_YAML: &str = include_str!("../config-default.yaml");

#[derive(Debug, Deserialize)]
pub(super) enum LogFormat {
    Hierarchy,
    Lines,
}

#[derive(Debug, Deserialize)]
pub(super) enum LogLevel {
    Debug,
    Info,
    Error,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Config {
    pub admin_emails: Vec<String>,

    pub log_format: LogFormat,
    pub log_level: LogLevel,

    pub smtp_server: String,

    pub smtp_port: u16,

    pub smtp_tls: bool,

    pub smtp_login: String,
    pub smtp_password: String,

    pub smtp_from: String,

    pub expire_soon_days: u16,

    pub ok_report_day: u8,

    pub no_cache_days_before_expire: i64,

    pub state_file: String,

    pub customers_file: String,
}

impl Config {
    #[allow(dead_code)]
    pub fn default() -> Self {
        return default_config().try_into().unwrap();
    }

    pub fn from_file(fname: &str) -> Result<Self> {
        let mut cfg = default_config();
        cfg.merge(config::File::with_name(fname))?;
        return Ok(cfg.try_into()?);
    }
}

fn default_config() -> ::config::Config {
    let mut settings = ::config::Config::new();
    let config_file = config::File::from_str(DEFAULT_CONFIG_YAML, config::FileFormat::Yaml);
    settings.merge(config_file).unwrap();
    return settings;
}
