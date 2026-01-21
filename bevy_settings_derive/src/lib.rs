use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, DeriveInput, Lit, Meta, parse_macro_input};

#[proc_macro_derive(SettingsGroup, attributes(settings))]
pub fn derive_settings_group(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    // 1. Parse #[settings("...")]
    let template = parse_settings_attribute(&input.attrs)
        .expect("Missing or invalid #[settings(\"...\")] attribute");
    // 2. Extract params from template string
    let params = extract_params(&template);
    // 3. Generate the implementation
    let expanded = quote! {
        // Implement bevy_paths::TypedPath
        impl bevy_paths::TypedPath for #name {
            const TEMPLATE: &'static str = #template;
            const PLACEHOLDERS: &'static [&'static str] = &[#(#params),*];
        }
        // Implement SettingsGroupTrait
        impl bevy_settings::SettingsGroupTrait for #name {
            fn path_params() -> &'static [&'static str] {
                &[#(#params),*]
            }
        }
    };
    TokenStream::from(expanded)
}

/// Parse #[settings("...")]
fn parse_settings_attribute(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("settings") {
            // Case 1: #[settings("path")]
            if let Meta::List(list) = &attr.meta {
                if let Ok(syn::Expr::Lit(syn::ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                })) = list.parse_args()
                {
                    return Some(lit_str.value());
                }
            }
            // Case 2: #[settings = "path"]
            if let Meta::NameValue(nv) = &attr.meta {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }) = &nv.value
                {
                    return Some(lit_str.value());
                }
            }
        }
    }
    None
}

/// Simple extraction of "{param}" placeholders
fn extract_params(template: &str) -> Vec<String> {
    let mut params = Vec::new();
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' {
            let mut param_name = String::new();
            while let Some(&next) = chars.peek() {
                if next == '}' {
                    chars.next(); // consume '}'
                    break;
                }
                param_name.push(chars.next().unwrap());
            }
            if !param_name.is_empty() {
                params.push(param_name);
            }
        }
    }
    params
}
