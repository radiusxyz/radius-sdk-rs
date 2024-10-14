use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::Parse,
    punctuated::{self, Punctuated},
    DeriveInput, Error, Ident, Meta, Result, Token, Type,
};

#[derive(Debug)]
pub struct Key {
    pub name: Ident,
    pub punctuation: Token![:],
    pub reference: Option<Token![ref]>,
    pub key_type: Type,
}

impl Parse for Key {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        Ok(Self {
            name: input.parse()?,
            punctuation: input.parse()?,
            reference: input.parse()?,
            key_type: input.parse()?,
        })
    }
}

#[derive(Debug)]
pub struct KeyAttributes {
    key_list: Punctuated<Key, Token![,]>,
}

impl Parse for KeyAttributes {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        Ok(Self {
            key_list: Punctuated::parse_terminated(input)?,
        })
    }
}

impl KeyAttributes {
    pub fn from_ast(ast: &DeriveInput) -> Result<Self> {
        let attribute = ast
            .attrs
            .iter()
            .find(|attribute| attribute.path().is_ident("kvstore_key"));

        match attribute {
            Some(attribute) => match &attribute.meta {
                Meta::List(meta_list) => {
                    let key_attributes = syn::parse2::<Self>(meta_list.tokens.to_token_stream())?;
                    return Ok(key_attributes);
                }
                others => return Err(Error::new_spanned(others, "key attributes must be a list.")),
            },
            None => {
                return Err(Error::new_spanned(
                    ast,
                    "`derive(Model)` requires 'kvstore_key' attribute.",
                ))
            }
        }
    }

    pub fn iter(&self) -> punctuated::Iter<'_, Key> {
        self.key_list.iter()
    }

    pub fn as_function_parameters(&self) -> TokenStream {
        let key_ident = self.key_list.iter().map(|key| &key.name);
        let key_punctuation = self.key_list.iter().map(|key| &key.punctuation);
        let key_reference = self.key_list.iter().map(|key| &key.reference);
        let key_type = self.key_list.iter().map(|key| &key.key_type);

        quote! {
            #(#key_ident #key_punctuation #key_reference #key_type,)*
        }
    }
}
