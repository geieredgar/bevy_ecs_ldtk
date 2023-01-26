use quote::quote;
use syn::Expr;

static FIELD_ATTRIBUTE_NAME: &str = "field";
static DEFAULT_VALUE_ATTRIBUTE_NAME: &str = "default_value";
static WITH_ATTRIBUTE_NAME: &str = "with";
static POST_ATTRIBUTE_NAME: &str = "post";
static PRE_ATTRIBUTE_NAME: &str = "pre";
static CONTEXT_ATTRIBUTE_NAME: &str = "context";
static NO_LIFETIME_ATTRIBUTE_NAME: &str = "no_lifetime";

pub fn expand_spawn_derive(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    let struct_name = &ast.ident;

    let fields = match &ast.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("Expected a struct with named fields."),
    };

    let context_path = match ast
        .attrs
        .iter()
        .find(|a| *a.path.get_ident().as_ref().unwrap() == CONTEXT_ATTRIBUTE_NAME)
    {
        Some(attribute) => expand_path_attribute(attribute, CONTEXT_ATTRIBUTE_NAME),
        None => panic!("Expected a #[context...] attribute on the struct"),
    };

    let context_type = if ast
        .attrs
        .iter()
        .any(|a| *a.path.get_ident().as_ref().unwrap() == NO_LIFETIME_ATTRIBUTE_NAME)
    {
        quote! { #context_path }
    } else {
        quote! { #context_path<'a> }
    };

    let mut field_constructions = Vec::new();
    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        if let Some(attribute) = field
            .attrs
            .iter()
            .find(|a| *a.path.get_ident().as_ref().unwrap() == DEFAULT_VALUE_ATTRIBUTE_NAME)
        {
            field_constructions.push(expand_default_value_attribute(
                attribute, field_name, field_type,
            ));
            continue;
        }

        let (source, wrapper) = match field
            .attrs
            .iter()
            .find(|a| *a.path.get_ident().as_ref().unwrap() == FIELD_ATTRIBUTE_NAME)
        {
            Some(attribute) => expand_field_attribute(attribute, field_name),
            None => (
                quote! {
                    (&mut context)
                },
                None,
            ),
        };

        let pre = match field
            .attrs
            .iter()
            .find(|a| *a.path.get_ident().as_ref().unwrap() == PRE_ATTRIBUTE_NAME)
        {
            Some(attribute) => {
                let path = expand_path_attribute(attribute, PRE_ATTRIBUTE_NAME);
                quote! {
                    #path(#source)
                }
            }
            None => quote! {
                #source.into()
            },
        };

        let with = match field
            .attrs
            .iter()
            .find(|a| *a.path.get_ident().as_ref().unwrap() == WITH_ATTRIBUTE_NAME)
        {
            Some(attribute) => {
                let path = expand_path_attribute(attribute, WITH_ATTRIBUTE_NAME);
                quote! {
                    #path(#pre)
                }
            }
            None => quote! {
                <#field_type as bevy_ecs_ldtk::prelude::Spawn>::spawn(#pre)
            },
        };

        let post = match field
            .attrs
            .iter()
            .find(|a| *a.path.get_ident().as_ref().unwrap() == POST_ATTRIBUTE_NAME)
        {
            Some(attribute) => {
                let path = expand_path_attribute(attribute, POST_ATTRIBUTE_NAME);
                quote! {
                    #path(#with)
                }
            }
            None => with,
        };

        let value = if let Some(wrapper) = wrapper {
            wrapper(post)
        } else {
            post
        };

        field_constructions.push(quote! {
            #field_name: #value,
        });
    }

    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let gen = quote! {
        impl #impl_generics bevy_ecs_ldtk::prelude::Spawn for #struct_name #ty_generics #where_clause {
            type Context<'a> = #context_type;

            fn spawn(mut context: Self::Context<'_>) -> Self {
                Self {
                    #(#field_constructions)*
                }
            }
        }
    };
    gen.into()
}

fn expand_path_attribute(attribute: &syn::Attribute, name: &str) -> syn::Path {
    match attribute
        .parse_meta()
        .expect(&format!("Cannot parse #[{name}...] attribute"))
    {
        syn::Meta::List(syn::MetaList { nested, .. }) if nested.len() == 1 => {
            match nested.first().unwrap() {
                syn::NestedMeta::Meta(syn::Meta::Path(path)) => path.clone(),
                _ => panic!("Expected function as the only argument of #[{name}(...)]"),
            }
        }
        _ => {
            panic!("#[{name}...] attribute should take the form #[{name}(path)]")
        }
    }
}

fn expand_default_value_attribute(
    attribute: &syn::Attribute,
    field_name: &syn::Ident,
    field_type: &syn::Type,
) -> proc_macro2::TokenStream {
    match attribute
        .parse_meta()
        .expect("Cannot parse #[default_value] attribute")
    {
        syn::Meta::Path(_) => {
            quote! { #field_name: <#field_type as std::default::Default>::default(), }
        }
        syn::Meta::NameValue(name_value) => match name_value {
            syn::MetaNameValue {
                lit: syn::Lit::Str(expr),
                ..
            } => {
                let expr = expr.parse::<Expr>().unwrap();
                quote! { #field_name: #expr, }
            }
            _ => panic!("#[default_value = ...] attribute only accepts string literals"),
        },
        _ => {
            panic!("#[default_value...] attribute should take the form #[default_value = \"expr\"] or #[default_value]")
        }
    }
}

fn expand_field_attribute(
    attribute: &syn::Attribute,
    field_name: &syn::Ident,
) -> (
    proc_macro2::TokenStream,
    Option<impl FnOnce(proc_macro2::TokenStream) -> proc_macro2::TokenStream>,
) {
    let (identifier, required) = match attribute
        .parse_meta()
        .expect("Cannot parse #[field] attribute")
    {
        syn::Meta::Path(_) => (field_name.to_string(), false),
        syn::Meta::NameValue(name_value) => match name_value {
            syn::MetaNameValue {
                lit: syn::Lit::Str(identifier),
                ..
            } => (identifier.value(), false),
            _ => panic!("#[field = ...] attribute only accepts string literals"),
        },
        syn::Meta::List(list) => {
            let mut identifier = None;
            let mut required = false;
            for meta in list.nested {
                match meta {
                    syn::NestedMeta::Meta(syn::Meta::Path(path)) if path.is_ident("required") && !required => {
                        required = true;
                    },
                    syn::NestedMeta::Lit(syn::Lit::Str(lit)) if identifier.is_none() => {
                        identifier = Some(lit.value());
                    }
                    _ => panic!("#[field(...)] attribute can contain at most one string literal and a 'required' entry")
                }
            }
            (
                identifier.unwrap_or_else(|| field_name.to_string()),
                required,
            )
        }
    };
    if required {
        let message = format!(
            "EntityInstance does not contain a FieldInstance with identifier \"{identifier}\""
        );
        (quote! { context.field(#identifier).expect(#message) }, None)
    } else {
        (
            quote! { context },
            Some(
                move |inner| quote! { context.field(#identifier).map(|context| #inner).unwrap_or_default() },
            ),
        )
    }
}
