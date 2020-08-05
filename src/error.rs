use serde_json::Error as JsonError;
use std::io::Error as IoError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to read config file: {0}")]
    ReadConfig(#[from] IoError),

    #[error("Failed to parse config file: {0}")]
    ParseConfig(#[from] JsonError),

    #[error("System not running")]
    SystemNotRunning,

    #[error("Configure '{name}' not found")]
    ConfigureNotFound { name: String },

    #[error("Trait '{name}' is not defined")]
    TraitNotDefined { name: String },

    #[error("Component '{name}' not found")]
    ComponentNotFound { name: String },

    #[error("Component '{component_name}' not implemented for '{trait_name}'")]
    NotImplemented {
        trait_name: String,
        component_name: String,
    },

    #[error("Component config '{name}' not found")]
    ComponentConfigNotFound { name: String },

    #[error("Component '{component_name}' missing propery '{property_name}'")]
    MissingProperty {
        component_name: String,
        property_name: String,
    },

    #[error("Invalid property '{property_name}' for component '{component_name}': {message}")]
    InvalidProperty {
        component_name: String,
        property_name: String,
        message: String,
    },

    #[error("Other error: {0}")]
    Other(anyhow::Error),
}

pub type Result<T> = ::std::result::Result<T, Error>;
