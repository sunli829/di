use crate::config::Config;
use crate::{Error, Injected, Registry, Result};
use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

pub struct Context {
    pub(crate) config: Arc<Config>,
    pub(crate) registry: Arc<Registry>,
    pub(crate) instances: HashMap<String, Box<dyn Any>>,
}

impl Context {
    fn downcast_component<T: Any + ?Sized>(
        instance: &Box<dyn Any>,
        component_name: &str,
    ) -> Result<Injected<T>> {
        match instance.downcast_ref::<Injected<T>>() {
            Some(injected) => Ok(injected.clone()),
            None => Err(Error::NotImplemented {
                trait_name: type_name::<T>().to_string(),
                component_name: component_name.to_string(),
            }),
        }
    }

    pub fn get<T: Any + ?Sized>(&mut self, name: &str) -> Result<Injected<T>> {
        match self.config.clone().components.get(name) {
            Some(component_config) => {
                if let Some(instance) = self.instances.get(name) {
                    return Self::downcast_component::<T>(instance, &component_config.name);
                }

                let components = self.registry.types.get(&TypeId::of::<T>()).ok_or_else(|| {
                    Error::TraitNotDefined {
                        name: type_name::<T>().to_string(),
                    }
                })?;
                match components.get(component_config.name.as_str()) {
                    Some(factory) => {
                        let instance = factory(self, &component_config.props)?;
                        self.instances.insert(name.to_string(), instance);
                        let instance = self.instances.get(name).unwrap();
                        Self::downcast_component::<T>(instance, &component_config.name)
                    }
                    None => Err(Error::ComponentNotFound {
                        name: name.to_string(),
                    }),
                }
            }
            None => Err(Error::ComponentConfigNotFound {
                name: name.to_string(),
            }),
        }
    }
}
