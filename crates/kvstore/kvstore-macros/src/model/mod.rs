mod attribute;
mod impl_block;

use attribute::*;
use impl_block::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, DeriveInput, Result};

pub fn expand_derive_model(input: &mut DeriveInput) -> Result<TokenStream> {
    let ident = &input.ident;
    let generics = &input.generics;
    let kvstore_attribute = KvStoreAttribute::from_ast(input)?;
    let mut generics_with_bounds = generics.clone();
    for param in generics_with_bounds.type_params_mut() {
        param.bounds.push(parse_quote!(Debug));
        param.bounds.push(parse_quote!(Serialize));
        param.bounds.push(parse_quote!(DeserializeOwned));
    }

    let id = const_id(ident);
    let put = fn_put(&kvstore_attribute);
    let get = fn_get(&kvstore_attribute);
    let get_or = fn_get_or(&kvstore_attribute);
    let get_mut = fn_get_mut(&kvstore_attribute);
    let get_mut_or = fn_get_mut_or(&kvstore_attribute);
    let apply = fn_apply(&kvstore_attribute);
    let delete = fn_delete(&kvstore_attribute);

    Ok(quote! {
        impl #generics_with_bounds #ident #generics {
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
