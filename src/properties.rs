use crate::code::{
    check_if_no_value_from_body_iter, check_signature_from_header, create_return_msg_from_header,
    get_member_from_header, get_string_from_body_iter, get_variant_from_body_iter,
    unknown_interface_from_header, unknown_member_from_header,
};
use crate::interface::Interface;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

fn create_get_code(interfaces: &[Interface]) -> Option<TokenStream> {
    let mut properties = Vec::new();
    for interface in interfaces {
        if let Some(code) = interface.create_get_code() {
            properties.push(code);
        }
    }

    if properties.is_empty() {
        None
    } else {
        let check_signature_from_header = check_signature_from_header("ss");
        let get_interface_from_body_iter = get_string_from_body_iter(&format_ident!("interface"));
        let get_property_from_body_iter = get_string_from_body_iter(&format_ident!("property"));
        let check_if_no_value_from_body_iter = check_if_no_value_from_body_iter();
        let unknown_interface_from_header = unknown_interface_from_header();
        let create_return_msg_from_header = create_return_msg_from_header();
        let code = quote!(
            "Get" => {
                #check_signature_from_header;
                #get_interface_from_body_iter;
                #get_property_from_body_iter;
                #check_if_no_value_from_body_iter;
                let value = match interface.as_ref() {
                    #(#properties)*
                    _ => #unknown_interface_from_header
                };
                let mut msg = #create_return_msg_from_header;
                msg.add_value(value);
                return dbus.send(msg);
            }
        );
        Some(code)
    }
}

fn create_get_all_code(interfaces: &[Interface]) -> Option<TokenStream> {
    let mut properties = Vec::new();
    for interface in interfaces {
        if let Some(code) = interface.create_get_all_code() {
            properties.push(code);
        }
    }

    if properties.is_empty() {
        None
    } else {
        let check_signature_from_header = check_signature_from_header("s");
        let get_interface_from_body_iter = get_string_from_body_iter(&format_ident!("interface"));
        let check_if_no_value_from_body_iter = check_if_no_value_from_body_iter();
        let create_return_msg_from_header = create_return_msg_from_header();
        let unknown_interface_from_header = unknown_interface_from_header();
        let code = quote!(
            "GetAll" => {
                #check_signature_from_header
                #get_interface_from_body_iter;
                #check_if_no_value_from_body_iter;
                let values = match interface.as_ref() {
                    #(#properties)*
                    _ => #unknown_interface_from_header
                };
                let values = dbus_message_parser::Value::Array(values, "v".to_string());
                let mut msg = #create_return_msg_from_header;
                msg.add_value(values);
                return dbus.send(msg);
            }
        );
        Some(code)
    }
}

fn create_set_code(interfaces: &[Interface]) -> Option<TokenStream> {
    let mut properties = Vec::new();
    for interface in interfaces {
        if let Some(code) = interface.create_set_code() {
            properties.push(code);
        }
    }

    if properties.is_empty() {
        None
    } else {
        let check_signature_from_header = check_signature_from_header("ssv");
        let get_interface_from_body_iter = get_string_from_body_iter(&format_ident!("interface"));
        let get_property_from_body_iter = get_string_from_body_iter(&format_ident!("property"));
        let get_value = get_variant_from_body_iter(&format_ident!("variant"));
        let check_if_no_value_from_body_iter = check_if_no_value_from_body_iter();
        let unknown_interface_from_header = unknown_interface_from_header();
        let create_return_msg_from_header = create_return_msg_from_header();
        let code = quote!(
            "Set" => {
                #check_signature_from_header;
                #get_interface_from_body_iter;
                #get_property_from_body_iter;
                #get_value;
                #check_if_no_value_from_body_iter;
                let i = *variant;
                match interface.as_ref() {
                    #(#properties)*
                    _ => #unknown_interface_from_header
                }
                let msg = #create_return_msg_from_header;
                return dbus.send(msg);
            }
        );
        Some(code)
    }
}

pub(super) fn create_properties_code(interfaces: &[Interface]) -> Option<TokenStream> {
    let get_member_from_header = get_member_from_header();
    let get = create_get_code(interfaces);
    let get_all = create_get_all_code(interfaces);
    let set = create_set_code(interfaces);
    let unknown_member_from_header = unknown_member_from_header();
    if get.is_none() && get_all.is_none() && set.is_none() {
        None
    } else {
        let code = quote! {
            "org.freedesktop.DBus.Properties" => {
                match #get_member_from_header {
                    #get
                    #get_all
                    #set
                    _ => #unknown_member_from_header
                }
            }
        };
        Some(code)
    }
}
