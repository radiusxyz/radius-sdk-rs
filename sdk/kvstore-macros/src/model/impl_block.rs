use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::model::attribute::KeyAttributes;

pub fn const_id(type_name: &Ident) -> TokenStream {
    quote! {
        const ID: &'static str = stringify!(#type_name);
    }
}

pub fn fn_put(key_attributes: &KeyAttributes) -> TokenStream {
    let parameters = key_attributes.as_function_parameters();
    let key_names = key_attributes.iter().map(|key| &key.name);

    quote! {
        pub fn put(&self, #parameters) -> std::result::Result<(), radius_sdk::kvstore::KvStoreError> {
            let key = &(Self::ID, #(#key_names,)*);

            radius_sdk::kvstore::kvstore()?.put(key, self)
        }
    }
}

pub fn fn_get(key_attributes: &KeyAttributes) -> TokenStream {
    let parameters = key_attributes.as_function_parameters();
    let key_names = key_attributes.iter().map(|key| &key.name);

    quote! {
        pub fn get(#parameters) -> std::result::Result<Self, radius_sdk::kvstore::KvStoreError> {
            let key = &(Self::ID, #(#key_names,)*);

            radius_sdk::kvstore::kvstore()?.get(key)
        }
    }
}

pub fn fn_get_or(key_attributes: &KeyAttributes) -> TokenStream {
    let parameters = key_attributes.as_function_parameters();
    let key_names = key_attributes.iter().map(|key| &key.name);

    quote! {
        pub fn get_or<F>(#parameters function: F) -> std::result::Result<Self, radius_sdk::kvstore::KvStoreError>
        where
            F: FnOnce() -> Self,
        {
            let key = &(Self::ID, #(#key_names,)*);

            radius_sdk::kvstore::kvstore()?.get_or(key, function)
        }
    }
}

pub fn fn_get_mut(key_attributes: &KeyAttributes) -> TokenStream {
    let parameters = key_attributes.as_function_parameters();
    let key_names = key_attributes.iter().map(|key| &key.name);

    quote! {
        pub fn get_mut(#parameters) -> std::result::Result<radius_sdk::kvstore::Lock<'static, Self>, radius_sdk::kvstore::KvStoreError> {
            let key = &(Self::ID, #(#key_names,)*);

            radius_sdk::kvstore::kvstore()?.get_mut(key)
        }
    }
}

pub fn fn_get_mut_or(key_attributes: &KeyAttributes) -> TokenStream {
    let parameters = key_attributes.as_function_parameters();
    let key_names = key_attributes.iter().map(|key| &key.name);

    quote! {
        pub fn get_mut_or<F>(#parameters function: F) -> std::result::Result<radius_sdk::kvstore::Lock<'static, Self>, radius_sdk::kvstore::KvStoreError>
        where
            F: FnOnce() -> Self,
        {
            let key = &(Self::ID, #(#key_names,)*);

            radius_sdk::kvstore::kvstore()?.get_mut_or(key, function)
        }
    }
}

pub fn fn_apply(key_attributes: &KeyAttributes) -> TokenStream {
    let parameters = key_attributes.as_function_parameters();
    let key_names = key_attributes.iter().map(|key| &key.name);

    quote! {
        pub fn apply<F>(#parameters operation: F) -> std::result::Result<(), radius_sdk::kvstore::KvStoreError>
        where
            F: FnOnce(&mut Self),
        {
            let key = &(Self::ID, #(#key_names,)*);

            radius_sdk::kvstore::kvstore()?.apply(key, |value: &mut radius_sdk::kvstore::Lock<'_, Self>| { operation(value) })
        }
    }
}

pub fn fn_delete(key_attributes: &KeyAttributes) -> TokenStream {
    let parameters = key_attributes.as_function_parameters();
    let key_names = key_attributes.iter().map(|key| &key.name);

    quote! {
        pub fn delete(#parameters) -> std::result::Result<(), radius_sdk::kvstore::KvStoreError> {
            let key = &(Self::ID, #(#key_names,)*);

            radius_sdk::kvstore::kvstore()?.delete(key)
        }
    }
}
