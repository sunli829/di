use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Error, Fields, GenericArgument, Meta, NestedMeta, Path,
    PathArguments, Result, Type,
};

struct ComponentArgs {
    name: Option<String>,
    interface: Path,
    init: Option<Path>,
}

enum ValueDefault {
    None,
    Default,
    DefaultCall(Path),
}

enum InjectArgs {
    InjectValue {
        name: Option<String>,
        default: ValueDefault,
    },
    InjectComponent {
        name: Option<String>,
    },
}

fn parse_component_args(input: &DeriveInput) -> Result<ComponentArgs> {
    let mut name = None;
    let mut interface = None;
    let mut init = None;

    for attr in &input.attrs {
        match attr.parse_meta()? {
            Meta::List(ls) if ls.path.is_ident("di") => {
                for meta in &ls.nested {
                    match meta {
                        NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("name") => {
                            if let syn::Lit::Str(lit) = &nv.lit {
                                name = Some(lit.value());
                            } else {
                                return Err(Error::new_spanned(
                                    &nv.lit,
                                    "Attribute 'name' should be a string.",
                                ));
                            }
                        }
                        NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("interface") => {
                            if let syn::Lit::Str(lit) = &nv.lit {
                                if let Ok(path) = syn::parse_str::<syn::Path>(&lit.value()) {
                                    interface = Some(path);
                                } else {
                                    return Err(Error::new_spanned(&lit, "Expect path"));
                                }
                            } else {
                                return Err(Error::new_spanned(
                                    &nv.lit,
                                    "Attribute 'type' should be a string.",
                                ));
                            }
                        }
                        NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("init") => {
                            if let syn::Lit::Str(lit) = &nv.lit {
                                if let Ok(path) = syn::parse_str::<syn::Path>(&lit.value()) {
                                    init = Some(path);
                                } else {
                                    return Err(Error::new_spanned(&lit, "Expect path"));
                                }
                            } else {
                                return Err(Error::new_spanned(
                                    &nv.lit,
                                    "Attribute 'init' should be a string.",
                                ));
                            }
                        }
                        _ => {}
                    }
                }
            }

            _ => {}
        }
    }

    Ok(ComponentArgs {
        name,
        interface: interface
            .ok_or_else(|| Error::new_spanned(input, "Missing 'interface' attribute"))?,
        init,
    })
}

fn parse_inject_args(attrs: &[Attribute]) -> Result<Option<InjectArgs>> {
    for attr in attrs {
        match attr.parse_meta()? {
            Meta::Path(path) if path.is_ident("value") => {
                return Ok(Some(InjectArgs::InjectValue {
                    name: None,
                    default: ValueDefault::None,
                }));
            }
            Meta::Path(path) if path.is_ident("inject") => {
                return Ok(Some(InjectArgs::InjectComponent { name: None }));
            }
            Meta::List(ls) if ls.path.is_ident("value") => {
                let mut name = None;
                let mut default = ValueDefault::None;

                for meta in &ls.nested {
                    match meta {
                        NestedMeta::Meta(Meta::Path(p)) if p.is_ident("default") => {
                            default = ValueDefault::Default;
                        }
                        NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("name") => {
                            if let syn::Lit::Str(lit) = &nv.lit {
                                name = Some(lit.value());
                            } else {
                                return Err(Error::new_spanned(
                                    &nv.lit,
                                    "Attribute 'name' should be a string.",
                                ));
                            }
                        }
                        NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("default") => {
                            if let syn::Lit::Str(lit) = &nv.lit {
                                if let Ok(path) = syn::parse_str::<syn::Path>(&lit.value()) {
                                    default = ValueDefault::DefaultCall(path);
                                } else {
                                    return Err(Error::new_spanned(&lit, "Expect ident"));
                                }
                            } else {
                                return Err(Error::new_spanned(
                                    &nv.lit,
                                    "Attribute 'default' should be a string.",
                                ));
                            }
                        }
                        _ => {}
                    }
                }

                return Ok(Some(InjectArgs::InjectValue { name, default }));
            }

            Meta::List(ls) if ls.path.is_ident("inject") => {
                let mut name = None;

                for meta in &ls.nested {
                    match meta {
                        NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("name") => {
                            if let syn::Lit::Str(lit) = &nv.lit {
                                name = Some(lit.value());
                            } else {
                                return Err(Error::new_spanned(
                                    &nv.lit,
                                    "Attribute 'name' should be a string.",
                                ));
                            }
                        }
                        _ => {}
                    }
                }

                return Ok(Some(InjectArgs::InjectComponent { name }));
            }

            _ => {}
        }
    }

    Ok(None)
}

pub fn generate(input: TokenStream) -> Result<TokenStream> {
    let input: DeriveInput = syn::parse2(input)?;
    let s = match &input.data {
        Data::Struct(s) => s,
        _ => return Err(Error::new_spanned(input, "It should be a struct")),
    };
    let component_args = parse_component_args(&input)?;
    let interface = &component_args.interface;
    let typename = &input.ident;
    let component_name = component_args.name.unwrap_or_else(|| typename.to_string());

    let mut set_props = Vec::new();
    let fields = match &s.fields {
        Fields::Named(fields) => fields,
        _ => return Err(Error::new_spanned(input, "All fields must be named.")),
    };

    for field in &fields.named {
        let field_ident = &field.ident;

        if let Some(inject_args) = parse_inject_args(&field.attrs)? {
            match inject_args {
                InjectArgs::InjectValue { name, default } => {
                    let property_name =
                        name.unwrap_or_else(|| field.ident.as_ref().unwrap().to_string());
                    let get_default = match default {
                        ValueDefault::None => {
                            quote! {
                                return Err(Error::MissingProperty {
                                    component_name: #component_name.to_string(),
                                    property_name: #property_name.to_string(),
                                });
                            }
                        }
                        ValueDefault::Default => {
                            quote! { Default::default() }
                        }
                        ValueDefault::DefaultCall(fun_name) => {
                            quote! { #fun_name() }
                        }
                    };

                    set_props.push(quote! {
                        #field_ident: match props.get(#property_name) {
                            Some(value) => {
                                di::serde_json::from_value(value.clone()).map_err(|err| {
                                    di::Error::InvalidProperty {
                                        component_name: #component_name.to_string(),
                                        property_name: #property_name.to_string(),
                                        message: err.to_string(),
                                    }
                                })?
                            }
                            None => { #get_default }
                        }
                    });
                }

                InjectArgs::InjectComponent { name } => {
                    let property_name =
                        name.unwrap_or_else(|| field.ident.as_ref().unwrap().to_string());
                    let mut interface_ty = None;

                    match &field.ty {
                        Type::Path(path)
                            if path.path.segments.last().unwrap().ident == "Injected" =>
                        {
                            if let PathArguments::AngleBracketed(args) =
                                &path.path.segments.last().unwrap().arguments
                            {
                                for arg in &args.args {
                                    if let GenericArgument::Type(ty) = arg {
                                        interface_ty = Some(ty);
                                        break;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }

                    let interface_ty = interface_ty
                        .ok_or_else(|| Error::new_spanned(&field.ty, "Invalid type"))?;

                    set_props.push(quote! {
                        #field_ident: match props.get(#property_name) {
                            Some(di::serde_json::Value::String(config)) => {
                                ctx.get::<#interface_ty>(config)?
                            }
                            Some(_) => {
                                return Err(di::Error::InvalidProperty {
                                    component_name: #component_name.to_string(),
                                    property_name: #property_name.to_string(),
                                    message: "Expect string".to_string(),
                                });
                            }
                            None => {
                                return Err(Error::MissingProperty {
                                    component_name: #component_name.to_string(),
                                    property_name: #property_name.to_string(),
                                });
                            }
                        }
                    });
                }
            }
        } else {
            set_props.push(quote! {
                #field_ident: Default::default()
            });
        }
    }

    let component_init = match component_args.init {
        Some(path) => quote! { #path(&mut component)? },
        None => quote! {},
    };

    let expanded = quote! {
        #[allow(unused_variables)]
        impl di::Component for #typename {
            type Interface = dyn #interface;

            fn name() -> &'static str { #component_name }

            fn create(ctx: &mut Context, props: &PropsMap) -> Result<::std::sync::Arc<Self::Interface>> {
                let mut component = Self {
                    #(#set_props),*
                };
                #component_init
                Ok(::std::sync::Arc::new(component))
            }
        }
    };

    Ok(expanded)
}
