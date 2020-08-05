use crate::config::Config;
use crate::context::Context;
use crate::{Component, Error, Registry};
use once_cell::sync::OnceCell;
use std::path::{Path, PathBuf};
use std::sync::Arc;

static SYSTEM: OnceCell<System> = OnceCell::new();

struct System {
    registry: Arc<Registry>,
    config: Arc<Config>,
}

pub struct SystemBuilder {
    config_file: Option<PathBuf>,
    registry: Registry,
}

impl SystemBuilder {
    pub fn new() -> SystemBuilder {
        SystemBuilder {
            config_file: None,
            registry: Default::default(),
        }
    }

    pub fn config_file(mut self, path: impl AsRef<Path>) -> Self {
        self.config_file = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn register<C: Component>(mut self) -> Self {
        self.registry.register::<C>();
        self
    }

    pub fn run<F, R>(self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let config = match self.config_file {
            Some(path) => Config::load(path).expect("Failed to load config file"),
            None => Default::default(),
        };

        SYSTEM
            .set(System {
                config: Arc::new(config),
                registry: Arc::new(self.registry),
            })
            .map_err(|_| "Failed to initialize system")
            .unwrap();
        f()
    }
}

fn get_system() -> &'static System {
    SYSTEM.get().ok_or_else(|| Error::SystemNotRunning).unwrap()
}

pub fn create_context() -> Context {
    Context {
        config: get_system().config.clone(),
        registry: get_system().registry.clone(),
        instances: Default::default(),
    }
}
