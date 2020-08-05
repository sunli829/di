use crate::component::{Component, Injected, PropsMap};
use crate::context::Context;
use crate::Result;
use std::any::{Any, TypeId};
use std::collections::HashMap;

type ComponentCreator = fn(&mut Context, &PropsMap) -> Result<Box<dyn Any>>;

#[derive(Default)]
pub struct Registry {
    pub(crate) types: HashMap<TypeId, HashMap<&'static str, ComponentCreator>>,
}

impl Registry {
    pub fn register<T: Component>(&mut self) {
        let component_name = T::name();
        let f: ComponentCreator = |ctx, props| Ok(Box::new(Injected(T::create(ctx, props)?)));
        self.types
            .entry(TypeId::of::<T::Interface>())
            .and_modify(|components| {
                components.insert(component_name, f);
            })
            .or_insert_with(|| {
                let mut components = HashMap::new();
                components.insert(component_name, f);
                components
            });
    }
}
