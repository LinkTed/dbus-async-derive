use crate::code::{
    get_member_from_header, unknown_member_from_header, unknown_property_from_header,
};
use crate::helper::{
    get_ident_from_path, get_lit_str_from_option_nested_meta, get_meta_list_from_nested_meta,
};
use crate::introspectable::Introspectable;
use crate::method::Method;
use crate::property::Property;
use crate::signal::Signal;
use proc_macro2::TokenStream;
use quote::quote;
use std::convert::TryFrom;
use syn::{Error as SynError, LitStr, MetaList};

pub(crate) struct Interface {
    name: LitStr,
    methods: Vec<Method>,
    properties: Vec<Property>,
    signals: Vec<Signal>,
}

impl Interface {
    pub(super) fn create_methods_code(&self) -> Option<TokenStream> {
        if self.methods.is_empty() {
            return None;
        }

        let get_member_from_header = get_member_from_header();
        let unknown_member_from_header = unknown_member_from_header();
        let name = &self.name;
        let mut methods = Vec::new();
        for method in &self.methods {
            methods.push(method.create_code());
        }
        let code = quote! {
            #name => {
                match #get_member_from_header {
                    #(#methods)*
                    _ => #unknown_member_from_header
                }
            }
        };
        Some(code)
    }

    pub(super) fn create_set_code(&self) -> Option<TokenStream> {
        if self.properties.is_empty() {
            return None;
        }

        let unknown_property_from_header = unknown_property_from_header();
        let name = &self.name;
        let mut properties = Vec::new();
        for property in &self.properties {
            properties.push(property.create_set_code());
        }
        let code = quote! {
            #name => {
                match property.as_ref() {
                    #(#properties)*
                    property => #unknown_property_from_header
                }
            }
        };
        Some(code)
    }

    pub(super) fn create_get_code(&self) -> Option<TokenStream> {
        if self.properties.is_empty() {
            return None;
        }

        let unknown_property_from_header = unknown_property_from_header();
        let name = &self.name;
        let mut properties = Vec::new();
        for property in &self.properties {
            properties.push(property.create_get_code());
        }
        let code = quote! {
            #name => {
                match property.as_ref() {
                    #(#properties)*
                    property => #unknown_property_from_header
                }
            }
        };
        Some(code)
    }

    pub(super) fn create_get_all_code(&self) -> Option<TokenStream> {
        if self.properties.is_empty() {
            return None;
        }

        let name = &self.name;
        let mut properties = Vec::new();
        for property in &self.properties {
            if let Some(get_all_property) = property.create_get_all_code() {
                properties.push(quote! {
                    {
                        o.push(#get_all_property);
                    }
                });
            }
        }
        let code = quote! {
            #name => {
                let mut o = Vec::new();
                #(#properties)*
                o
            }
        };
        Some(code)
    }
}

impl Introspectable for Interface {
    fn to_introspect(&self, xml: &mut String) {
        *xml += &format!("  <interface name=\"{}\">\n", self.name.value());
        for method in &self.methods {
            method.to_introspect(xml);
        }
        for property in &self.properties {
            property.to_introspect(xml);
        }
        for signal in &self.signals {
            signal.to_introspect(xml);
        }
        *xml += "  </interface>\n";
    }
}

impl TryFrom<&MetaList> for Interface {
    type Error = SynError;

    fn try_from(meta_list: &MetaList) -> Result<Self, Self::Error> {
        let meta_list_type = get_ident_from_path(&meta_list.path)?;
        if meta_list_type != "interface" {
            return Err(SynError::new(
                meta_list_type.span(),
                "excepted \"interface\"",
            ));
        }
        // Get the first argument. It is the name of the interface
        let nested_iter = &mut meta_list.nested.iter();
        // Get the interface name
        let name = get_lit_str_from_option_nested_meta(nested_iter.next())?;

        let mut methods = Vec::new();
        let mut properties = Vec::new();
        let mut signals = Vec::new();
        for nested_meta in nested_iter {
            let meta_list = get_meta_list_from_nested_meta(nested_meta)?;
            let ident = get_ident_from_path(&meta_list.path)?;
            match ident.to_string().as_ref() {
                "method" => {
                    let method = Method::try_from(meta_list)?;
                    methods.push(method);
                }
                "property" => {
                    let property = Property::try_from(meta_list)?;
                    properties.push(property);
                }
                "signal" => {
                    let signal = Signal::try_from(meta_list)?;
                    signals.push(signal);
                }
                attribute => {
                    return Err(SynError::new(
                        ident.span(),
                        format!("Unknown attribute: {}", attribute),
                    ))
                }
            }
        }

        Ok(Interface {
            name,
            methods,
            properties,
            signals,
        })
    }
}
