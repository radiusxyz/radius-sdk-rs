use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{discouraged::AnyDelimiter, Parse},
    punctuated::{self, Punctuated},
    DeriveInput, Error, Ident, Meta, Path, Result, Token, Type,
};

#[derive(Debug)]
pub struct KvStoreAttribute {
    path_attribute: PathAttribute,
    key_attribute: Option<KeyAttribute>,
}

impl KvStoreAttribute {
    pub fn from_ast(ast: &DeriveInput) -> Result<Self> {
        let mut path_attribute: Option<PathAttribute> = None;
        let mut key_attribute: Option<KeyAttribute> = None;

        for attribute in ast.attrs.iter() {
            if attribute.path().is_ident("kvstore") {
                match &attribute.meta {
                    Meta::List(meta_list) => {
                        let attribute =
                            syn::parse2::<AttributeType>(meta_list.tokens.to_token_stream())?;
                        match attribute {
                            AttributeType::Path(path) => {
                                if path_attribute.is_some() {
                                    return Err(Error::new_spanned(
                                        meta_list,
                                        "Attribute path already exists.",
                                    ));
                                }
                                path_attribute = Some(path);
                            }
                            AttributeType::Key(key) => {
                                if key_attribute.is_some() {
                                    return Err(Error::new_spanned(
                                        meta_list,
                                        "Attribute key already exists.",
                                    ));
                                }
                                key_attribute = Some(key);
                            }
                        }
                    }
                    others => return Err(Error::new_spanned(others, "Expect kvstore(token)")),
                }
            }
        }

        if path_attribute.is_none() {
            let default_path = quote!(radius_sdk::kvstore);
            let default_path: PathAttribute = syn::parse2(default_path)?;
            path_attribute = Some(default_path);
        }

        Ok(Self {
            path_attribute: path_attribute.unwrap(),
            key_attribute,
        })
    }

    pub fn path(&self) -> &Path {
        self.path_attribute.path()
    }

    pub fn key_attribute(&self) -> Option<&KeyAttribute> {
        self.key_attribute.as_ref()
    }
}

#[derive(Debug)]
pub enum AttributeType {
    Path(PathAttribute),
    Key(KeyAttribute),
}

impl Parse for AttributeType {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        match ident.to_string().as_str() {
            "path" => {
                let _punctuation: Token![=] = input.parse()?;
                let tokens: TokenStream = input.parse()?;
                let path_attribute = syn::parse2::<PathAttribute>(tokens)?;

                Ok(Self::Path(path_attribute))
            }
            "key" => {
                let tokens: TokenStream = input.parse()?;
                let key_attribute = syn::parse2::<KeyAttribute>(tokens)?;

                Ok(Self::Key(key_attribute))
            }
            _others => Err(Error::new_spanned(ident, "Must be 'path' or 'key'")),
        }
    }
}

#[derive(Debug)]
#[allow(unused)]
pub struct PathAttribute {
    // punctuation: Token![=],
    path: Path,
}

impl Parse for PathAttribute {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        Ok(Self {
            // punctuation: input.parse()?,
            path: input.parse()?,
        })
    }
}

impl PathAttribute {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Debug)]
pub struct KeyAttribute {
    key_list: Punctuated<Key, Token![,]>,
}

impl Parse for KeyAttribute {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let (_delimiter, _span, buffer) = input.parse_any_delimiter()?;

        Ok(Self {
            key_list: Punctuated::parse_terminated(&buffer)?,
        })
    }
}

impl KeyAttribute {
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
