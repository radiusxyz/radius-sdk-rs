mod model;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Error};

#[proc_macro_derive(Model, attributes(kvstore))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    model::expand_derive_model(&mut input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn test_basic_struct() {
        let input: syn::DeriveInput = parse_quote! {
            #[derive(Model)]
            #[kvstore(key(id: u64))]
            struct TestStruct {
                id: u64,
                name: String,
            }
        };

        let output = crate::model::expand_derive_model(&mut input.clone()).unwrap();
        println!("Generated code: {}", output);
    }

    #[test]
    fn test_generic_struct() {
        let input: syn::DeriveInput = parse_quote! {
            #[derive(Model)]
            #[kvstore(key(session_id: SessionId, address: Address))]
            struct PartialKeySubmission<Signature: Debug, Address> {
                signature: Signature,
                payload: PartialKeyPayload<Address>,
            }
        };

        let output = crate::model::expand_derive_model(&mut input.clone()).unwrap();
        println!("Generated code: {}", output);
    }
}
