mod attribute;
mod impl_block;

use attribute::*;
use impl_block::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Result};

pub fn expand_derive_model(input: &mut DeriveInput) -> Result<TokenStream> {
    let ident = &input.ident;
    let key_attributes = KeyAttributes::from_ast(input)?;

    let id = const_id(&ident);
    let put = fn_put(&key_attributes);
    let get = fn_get(&key_attributes);
    let get_or = fn_get_or(&key_attributes);
    let get_mut = fn_get_mut(&key_attributes);
    let get_mut_or = fn_get_mut_or(&key_attributes);
    let apply = fn_apply(&key_attributes);
    let delete = fn_delete(&key_attributes);

    Ok(quote! {
        impl #ident {
            #id
            #put
            #get
            #get_or
            #get_mut
            #get_mut_or
            #apply
            #delete
        }
    })
}
