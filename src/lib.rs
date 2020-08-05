mod component;
mod config;
mod context;
mod error;
mod registry;
mod system;

#[doc(hidden)]
pub use serde_json;

pub use component::{Component, Injected, PropsMap};
pub use context::Context;
pub use error::{Error, Result};
pub use registry::Registry;
pub use di_derive::Component;
pub use system::{create_context, SystemBuilder};
