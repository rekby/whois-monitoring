use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    WhoisError(whois2::Error),
    IoError(std::io::Error),
    SerdeError(serde_yaml::Error),
    ChronoFormatParseError(chrono::ParseError),
    CanFindWhoisField,
    ConfigError(::config::ConfigError),
    LettreEmailError(lettre_email::error::Error),
    LettreSmtpError(lettre::smtp::error::Error),
}

use Error::*;

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChronoFormatParseError(err) => Display::fmt(err, f),
            WhoisError(err) => Display::fmt(err, f),
            IoError(err) => Display::fmt(err, f),
            SerdeError(err) => Display::fmt(err, f),
            CanFindWhoisField => f.write_str("Can't find whois field"),
            ConfigError(err) => Display::fmt(err, f),
            LettreEmailError(err) => Display::fmt(err, f),
            LettreSmtpError(err) => Display::fmt(err, f),
        }
    }
}

impl From<::config::ConfigError> for Error {
    fn from(err: ::config::ConfigError) -> Error {
        ConfigError(err)
    }
}

impl From<chrono::format::ParseError> for Error {
    fn from(err: chrono::format::ParseError) -> Error {
        ChronoFormatParseError(err)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Error {
        SerdeError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        IoError(err)
    }
}

impl From<lettre::smtp::error::Error> for Error {
    fn from(err: lettre::smtp::error::Error) -> Self {
        LettreSmtpError(err)
    }
}

impl From<lettre_email::error::Error> for Error {
    fn from(err: lettre_email::error::Error) -> Self {
        LettreEmailError(err)
    }
}

impl From<whois2::Error> for Error {
    fn from(err: whois2::Error) -> Error {
        WhoisError(err)
    }
}
