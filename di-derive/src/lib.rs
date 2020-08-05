extern crate proc_macro;

mod component;

use proc_macro::TokenStream;

#[proc_macro_derive(Component, attributes(di, value, inject))]
pub fn derive_component(input: TokenStream) -> TokenStream {
    match component::generate(input.into()) {
        Ok(stream) => stream.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
