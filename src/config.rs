use crate::{PropsMap, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Default)]
pub struct ComponentConfig {
    pub name: String,
    #[serde(default)]
    pub props: PropsMap,
}

#[derive(Deserialize, Default)]
pub struct Config {
    pub components: HashMap<String, ComponentConfig>,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        Ok(serde_json::from_reader(
            fs::OpenOptions::new().read(true).open(path)?,
        )?)
    }
}
