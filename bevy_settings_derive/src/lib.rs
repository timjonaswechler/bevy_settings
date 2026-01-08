use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, DeriveInput, Lit, Meta};

#[proc_macro_derive(SettingsGroup, attributes(settings))]
pub fn derive_settings_group(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // 1. Parse #[settings(file = "...")]
    let template = parse_settings_attribute(&input.attrs)
        .expect("Missing or invalid #[settings(file = \"...\")] attribute");

    // 2. Extract params from template string (e.g. "{id}" -> "id")
    let params = extract_params(&template);

    // 3. Generate the implementation
    let expanded = quote! {
        // Implement SettingsGroup
        impl bevy_settings::SettingsGroup for #name {
            fn path_params() -> &'static [&'static str] {
                &[#(#params),*]
            }
        }

        // Also implement TypedPath automatically (from bevy_paths)
        impl bevy_paths::TypedPath for #name {
            fn template() -> &'static str {
                #template
            }
        }
    };

    TokenStream::from(expanded)
}

fn parse_settings_attribute(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("settings") {
            // Case A: #[settings(file = "...")]
            if let Meta::List(list) = &attr.meta {
                // We need to parse the nested key-value pairs manually or use syn helpers
                // But syn::Meta::List doesn't easily give key-values without `syn` features "extra-traits" sometimes
                // Let's try parsing tokens as MetaNameValue if possible, or iterate
                // Ideally we want to parse `file = "..."` inside the parens.

                // Simplified approach: convert tokens to string and regex/find? No, too fragile.
                // Better: Use `syn::parse2` to parse the content as `Meta`

                // For now, let's support the simple case where the content is just `file = "..."`
                // Actually, standard practice is comma separated Meta.

                // Let's try to parse the nested meta
                let nested_metas = list
                    .parse_args_with(
                        syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
                    )
                    .ok()?;

                for meta in nested_metas {
                    if let Meta::NameValue(nv) = meta {
                        if nv.path.is_ident("file") {
                            if let syn::Expr::Lit(expr_lit) = &nv.value {
                                if let Lit::Str(lit_str) = &expr_lit.lit {
                                    return Some(lit_str.value());
                                }
                            }
                        }
                    }
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
