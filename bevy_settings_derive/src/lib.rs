use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive macro for Settings trait
/// 
/// This macro implements the Settings trait for a struct, enabling it to be:
/// - Used as a Bevy resource
/// - Serialized/deserialized to JSON or binary format
/// - Managed with default values and delta persistence
///
/// # Example
/// ```ignore
/// use bevy_settings::Settings;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Settings, Serialize, Deserialize, Default, Clone)]
/// struct GameSettings {
///     volume: f32,
///     resolution: (u32, u32),
/// }
/// ```
#[proc_macro_derive(Settings)]
pub fn derive_settings(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    let expanded = quote! {
        impl bevy_settings::Settings for #name {
            fn type_name() -> &'static str {
                stringify!(#name)
            }
        }
    };
    
    TokenStream::from(expanded)
}
