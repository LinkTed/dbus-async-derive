#![recursion_limit = "256"]
extern crate proc_macro;

mod code;
mod helper;
mod interface;
mod introspectable;
mod method;
mod properties;
mod property;
mod signal;
mod signature;

use crate::code::{get_interface_from_header, unknown_interface_from_header};
use crate::helper::{get_ident_from_path, get_meta_list_from_meta};
use crate::interface::Interface;
use crate::introspectable::{create_introspectable_code, parse_introspectable};
use crate::properties::create_properties_code;
use proc_macro::TokenStream;
use quote::quote;
use std::convert::TryFrom;
use syn::{parse_macro_input, DeriveInput, Error as SynError, Result as SynResult};

/// Try to derive
fn try_derive(ast: DeriveInput) -> SynResult<TokenStream> {
    let struct_name = ast.ident;
    let mut introspectable = None;
    let mut interfaces = Vec::new();
    for attribute in ast.attrs {
        let meta = attribute.parse_meta()?;
        let meta_list = get_meta_list_from_meta(&meta)?;
        let meta_list_type = get_ident_from_path(&meta_list.path)?;
        match meta_list_type.to_string().as_ref() {
            "interface" => interfaces.push(Interface::try_from(meta_list)?),
            "introspectable" => {
                if introspectable.is_some() {
                    return Err(SynError::new(
                        meta_list_type.span(),
                        "Introspectable is defined multiple times",
                    ));
                } else {
                    let boolean = parse_introspectable(meta_list)?;
                    introspectable = Some(boolean);
                }
            }
            attribute => {
                return Err(SynError::new(
                    meta_list_type.span(),
                    format!("Unknown attribute: {}", attribute),
                ))
            }
        }
    }

    let introspectable = if let Some(introspectable) = introspectable {
        introspectable
    } else {
        true
    };

    let mut interfaces_code = Vec::new();

    let have_properties = match create_properties_code(&interfaces) {
        Some(code) => {
            interfaces_code.push(code);
            true
        }
        None => false,
    };

    if introspectable {
        interfaces_code.push(create_introspectable_code(&interfaces, have_properties));
    }

    for interface in &interfaces {
        if let Some(code) = interface.create_methods_code() {
            interfaces_code.push(code);
        }
    }

    let get_interface_from_header = get_interface_from_header();
    let unknown_interface_from_header = unknown_interface_from_header();
    let code = quote! {
        #[async_trait::async_trait]
        impl dbus_async::Handler for #struct_name {
            async fn handle(&mut self, dbus: &dbus_async::DBus, msg: dbus_message_parser::Message) -> dbus_async::DBusResult<()> {
                if msg.get_type() != dbus_message_parser::MessageType::MethodCall {
                    return Ok(())
                }
                let (header, body) = msg.split();
                let mut body_iter = body.into_iter();
                match #get_interface_from_header {
                    #(#interfaces_code)*
                    _ => #unknown_interface_from_header
                }
            }
        }
    };
    Ok(code.into())
}

/// The derive method.
#[proc_macro_derive(Handler, attributes(interface, introspectable))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    match try_derive(ast) {
        Ok(token) => token,
        Err(e) => e.to_compile_error().into(),
    }
}
