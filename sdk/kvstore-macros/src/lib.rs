mod model;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Error};

#[proc_macro_derive(Model, attributes(key))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    model::expand_derive_model(&mut input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
