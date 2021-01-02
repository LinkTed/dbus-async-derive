use crate::code::check_result;
use crate::helper::{
    get_ident_from_path, get_lit_str_from_lit, get_lit_str_from_option_nested_meta,
    get_meta_name_value_from_nested_meta,
};
use crate::introspectable::Introspectable;
use crate::signature::SignatureIterator;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::convert::TryFrom;
use syn::spanned::Spanned;
use syn::{Error as SynError, LitStr, MetaList, NestedMeta, Result as SynResult};

pub(super) struct Property {
    name: LitStr,
    get: Option<Ident>,
    set: Option<Ident>,
    signature: (String, TokenStream, TokenStream, TokenStream),
}

fn create_property_code(
    nested_meta: &NestedMeta,
    get: &mut Option<Ident>,
    set: &mut Option<Ident>,
) -> SynResult<()> {
    let meta_name_value = get_meta_name_value_from_nested_meta(nested_meta)?;
    let function = get_ident_from_path(&meta_name_value.path)?;
    let operation = get_lit_str_from_lit(&meta_name_value.lit)?;
    match operation.value().as_str() {
        "get" => {
            if get.is_some() {
                return Err(SynError::new(operation.span(), "get is defined twice"));
            }
            *get = Some(function);
            Ok(())
        }
        "set" => {
            if set.is_some() {
                return Err(SynError::new(operation.span(), "set is defined twice"));
            }
            *set = Some(function);
            Ok(())
        }
        x => Err(SynError::new(
            operation.span(),
            &format!("excepted \"get\" or \"set\" got {}", x),
        )),
    }
}

impl Property {
    pub(super) fn create_get_code(&self) -> TokenStream {
        let name = &self.name;
        if let Some(function) = &self.get {
            let (_, _, _, rust_to_value) = &self.signature;
            let check_result = check_result();
            quote! {
                #name => {
                    let result = self.#function(&dbus, &header).await;
                    let i = #check_result;
                    let v = std::boxed::Box::new(#rust_to_value);
                    dbus_message_parser::Value::Variant(v)
                }
            }
        } else {
            quote! {
                #name => {
                    let msg = header.error(
                        std::convert::TryFrom::try_from("org.freedesktop.DBus.Error.Property".to_string()).unwrap(),
                        "This property is write only".to_string());
                    return dbus.send(msg);
                }
            }
        }
    }

    pub(super) fn create_get_all_code(&self) -> Option<TokenStream> {
        if let Some(function) = &self.get {
            let (_, _, _, rust_to_value) = &self.signature;
            let check_result = check_result();
            let code = quote! {
                {
                    let result = self.#function(&dbus, &header).await;
                    let i = #check_result;
                    let v = std::boxed::Box::new(#rust_to_value);
                    dbus_message_parser::Value::Variant(v)
                }
            };
            Some(code)
        } else {
            None
        }
    }

    pub(super) fn create_set_code(&self) -> TokenStream {
        let name = &self.name;
        if let Some(function) = &self.set {
            let (_, rust_type, value_to_rust, _) = &self.signature;
            let check_result = check_result();
            quote! {
                #name => {
                    let value: #rust_type = #value_to_rust;
                    let result = self.#function(&dbus, &header, value).await;
                    #check_result;
                }
            }
        } else {
            quote! {
                #name => {
                    let msg = header.error(
                        std::convert::TryFrom::try_from("org.freedesktop.DBus.Error.Property".to_string()).unwrap(),
                        "This property is read only".to_string());
                    return dbus.send(msg);
                }
            }
        }
    }
}

impl Introspectable for Property {
    fn to_introspect(&self, xml: &mut String) {
        let mut access = String::new();
        if self.get.is_some() {
            access += "read";
        }

        if self.set.is_some() {
            access += "write";
        }

        *xml += &format!(
            "    <property type=\"{}\" name=\"{}\" access=\"{}\"/>\n",
            self.signature.0,
            self.name.value(),
            access
        );
    }
}

impl TryFrom<&MetaList> for Property {
    type Error = SynError;

    fn try_from(meta_list: &MetaList) -> Result<Self, Self::Error> {
        let mut get = None;
        let mut set = None;

        // Get the ident and check if it is equal "property"
        let meta_list_type = get_ident_from_path(&meta_list.path)?;
        if meta_list_type != "property" {
            return Err(SynError::new(
                meta_list_type.span(),
                "excepted \"property\"",
            ));
        }
        let nested_iter = &mut meta_list.nested.iter();
        // Get the name of the property
        let name = get_lit_str_from_option_nested_meta(nested_iter.next())?;

        // Get the signature
        let signature = get_lit_str_from_option_nested_meta(nested_iter.next())?;
        let mut signature_iter = SignatureIterator::from(&signature);
        let signature = if let Some(signature) = signature_iter.next() {
            signature?
        } else {
            return Err(SynError::new(
                signature.span(),
                "excepted only one signature type",
            ));
        };

        if signature_iter.next().is_some() {
            return Err(SynError::new(
                meta_list.span(),
                "excepted only one signature type",
            ));
        }

        if let Some(nested_meta) = nested_iter.next() {
            create_property_code(nested_meta, &mut get, &mut set)?;
        } else {
            return Err(SynError::new(
                meta_list.nested.span(),
                "no \"get\" or \"set\" function defined",
            ));
        }

        // Check if there is another attribute
        if let Some(nested_meta) = nested_iter.next() {
            // Create the code for the forth attribute
            create_property_code(nested_meta, &mut get, &mut set)?;
        }

        if nested_iter.next().is_some() {
            return Err(SynError::new(meta_list.span(), "too many arguments"));
        }

        Ok(Property {
            name,
            get,
            set,
            signature,
        })
    }
}
