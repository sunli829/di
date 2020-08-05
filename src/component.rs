use crate::context::Context;
use crate::Result;
use std::ops::Deref;
use std::sync::Arc;

pub type PropsMap = serde_json::Map<String, serde_json::Value>;

pub trait Component {
    type Interface: 'static + ?Sized;

    fn name() -> &'static str;

    fn create(ctx: &mut Context, props: &PropsMap) -> Result<Arc<Self::Interface>>;
}

pub struct Injected<T: ?Sized>(pub(crate) Arc<T>);

impl<T: ?Sized> Clone for Injected<T> {
    fn clone(&self) -> Self {
        Injected(self.0.clone())
    }
}

impl<T: ?Sized> Deref for Injected<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}
