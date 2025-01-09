mod attribute;
mod impl_block;

use attribute::*;
use impl_block::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Result};

pub fn expand_derive_model(input: &mut DeriveInput) -> Result<TokenStream> {
    let ident = &input.ident;
    let kvstore_attribute = KvStoreAttribute::from_ast(input)?;

    let id = const_id(ident);
    let put = fn_put(&kvstore_attribute);
    let get = fn_get(&kvstore_attribute);
    let get_or = fn_get_or(&kvstore_attribute);
    let get_mut = fn_get_mut(&kvstore_attribute);
    let get_mut_or = fn_get_mut_or(&kvstore_attribute);
    let apply = fn_apply(&kvstore_attribute);
    let delete = fn_delete(&kvstore_attribute);

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
