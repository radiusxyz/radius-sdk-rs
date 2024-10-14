use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::model::attribute::KvStoreAttribute;

pub fn const_id(type_name: &Ident) -> TokenStream {
    quote! {
        const ID: &'static str = stringify!(#type_name);
    }
}

pub fn fn_put(kvstore_attribute: &KvStoreAttribute) -> Option<TokenStream> {
    if let Some(key_attribute) = kvstore_attribute.key_attribute() {
        let parameters = key_attribute.as_function_parameters();
        let key_names = key_attribute.iter().map(|key| &key.name);
        let path = kvstore_attribute.path();

        Some(quote! {
            pub fn put(&self, #parameters) -> std::result::Result<(), #path::KvStoreError> {
                let key = &(Self::ID, #(#key_names,)*);

                radius_sdk::kvstore::kvstore()?.put(key, self)
            }
        })
    } else {
        None
    }
}

pub fn fn_get(kvstore_attribute: &KvStoreAttribute) -> Option<TokenStream> {
    if let Some(key_attribute) = kvstore_attribute.key_attribute() {
        let parameters = key_attribute.as_function_parameters();
        let key_names = key_attribute.iter().map(|key| &key.name);

        Some(quote! {
            pub fn get(#parameters) -> std::result::Result<Self, radius_sdk::kvstore::KvStoreError> {
                let key = &(Self::ID, #(#key_names,)*);

                radius_sdk::kvstore::kvstore()?.get(key)
            }
        })
    } else {
        None
    }
}

pub fn fn_get_or(kvstore_attribute: &KvStoreAttribute) -> Option<TokenStream> {
    if let Some(key_attribute) = kvstore_attribute.key_attribute() {
        let parameters = key_attribute.as_function_parameters();
        let key_names = key_attribute.iter().map(|key| &key.name);

        Some(quote! {
            pub fn get_or<F>(#parameters function: F) -> std::result::Result<Self, radius_sdk::kvstore::KvStoreError>
            where
                F: FnOnce() -> Self,
            {
                let key = &(Self::ID, #(#key_names,)*);

                radius_sdk::kvstore::kvstore()?.get_or(key, function)
            }
        })
    } else {
        None
    }
}

pub fn fn_get_mut(kvstore_attribute: &KvStoreAttribute) -> Option<TokenStream> {
    if let Some(key_attribute) = kvstore_attribute.key_attribute() {
        let parameters = key_attribute.as_function_parameters();
        let key_names = key_attribute.iter().map(|key| &key.name);

        Some(quote! {
            pub fn get_mut(#parameters) -> std::result::Result<radius_sdk::kvstore::Lock<'static, Self>, radius_sdk::kvstore::KvStoreError> {
                let key = &(Self::ID, #(#key_names,)*);

                radius_sdk::kvstore::kvstore()?.get_mut(key)
            }
        })
    } else {
        None
    }
}

pub fn fn_get_mut_or(kvstore_attribute: &KvStoreAttribute) -> Option<TokenStream> {
    if let Some(key_attribute) = kvstore_attribute.key_attribute() {
        let parameters = key_attribute.as_function_parameters();
        let key_names = key_attribute.iter().map(|key| &key.name);

        Some(quote! {
            pub fn get_mut_or<F>(#parameters function: F) -> std::result::Result<radius_sdk::kvstore::Lock<'static, Self>, radius_sdk::kvstore::KvStoreError>
            where
                F: FnOnce() -> Self,
            {
                let key = &(Self::ID, #(#key_names,)*);

                radius_sdk::kvstore::kvstore()?.get_mut_or(key, function)
            }
        })
    } else {
        None
    }
}

pub fn fn_apply(kvstore_attribute: &KvStoreAttribute) -> Option<TokenStream> {
    if let Some(key_attribute) = kvstore_attribute.key_attribute() {
        let parameters = key_attribute.as_function_parameters();
        let key_names = key_attribute.iter().map(|key| &key.name);

        Some(quote! {
            pub fn apply<F>(#parameters operation: F) -> std::result::Result<(), radius_sdk::kvstore::KvStoreError>
            where
                F: FnOnce(&mut Self),
            {
                let key = &(Self::ID, #(#key_names,)*);

                radius_sdk::kvstore::kvstore()?.apply(key, |value: &mut radius_sdk::kvstore::Lock<'_, Self>| { operation(value) })
            }
        })
    } else {
        None
    }
}

pub fn fn_delete(kvstore_attribute: &KvStoreAttribute) -> Option<TokenStream> {
    if let Some(key_attribute) = kvstore_attribute.key_attribute() {
        let parameters = key_attribute.as_function_parameters();
        let key_names = key_attribute.iter().map(|key| &key.name);

        Some(quote! {
            pub fn delete(#parameters) -> std::result::Result<(), radius_sdk::kvstore::KvStoreError> {
                let key = &(Self::ID, #(#key_names,)*);

                radius_sdk::kvstore::kvstore()?.delete(key)
            }
        })
    } else {
        None
    }
}
