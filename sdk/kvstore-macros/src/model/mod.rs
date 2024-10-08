mod attribute;

use attribute::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Result};

pub fn expand_derive_model(input: &mut DeriveInput) -> Result<TokenStream> {
    let ident = &input.ident;
    let key_attributes = KeyAttributes::from_ast(input)?;

    let put = fn_put(&key_attributes);
    let get = fn_get(&key_attributes);

    Ok(quote! {
        impl #ident {
            pub const ID: &'static str = stringify!(#ident);

            #put
            #get
        }
    })
}

pub fn fn_put(key_attributes: &KeyAttributes) -> TokenStream {
    let parameters = key_attributes.as_function_parameters();
    let key_names = key_attributes.iter().map(|key| &key.name);

    quote! {
        pub fn put(&self, #parameters) {
            let id = &(Self::ID, #(#key_names,)*);
        }
    }
}

pub fn fn_get(key_attributes: &KeyAttributes) -> TokenStream {
    let parameters = key_attributes.as_function_parameters();
    let key_names = key_attributes.iter().map(|key| &key.name);

    quote! {
        pub fn get(#parameters) {
            let id = &(Self::ID, #(#key_names,)*);
        }
    }
}
