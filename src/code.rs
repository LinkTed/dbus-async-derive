use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub(super) fn check_result() -> TokenStream {
    quote! {
        {
            match result {
                Ok(r) => r,
                Err((name, message)) => {
                    let msg = header.error(name, message);
                    return dbus.send(msg);
                }
            }
        }
    }
}

pub(super) fn invalid_args() -> TokenStream {
    quote! {
        let msg = header.invalid_args(text);
        return dbus.send(msg);
    }
}

pub(super) fn missing_value(excepted: &str) -> TokenStream {
    let text = format!("signature mismatch: excepted {}", excepted);
    let invalid_args = invalid_args();
    quote! {
        let text = #text.to_string();
        #invalid_args;
    }
}

pub(super) fn invalid_args_signature(excepted: &str) -> TokenStream {
    let text = format!("signature mismatch: excepted {} got {{}}", excepted);
    let invalid_args = invalid_args();
    quote! {
        let text = format!(#text, signature);
        #invalid_args
    }
}

pub(super) fn check_signature(excepted: &str) -> TokenStream {
    let invalid_args_signature = invalid_args_signature(excepted);
    quote! {
        if #excepted != signature {
            #invalid_args_signature
        }
    }
}

pub(super) fn check_signature_from_header(excepted: &str) -> TokenStream {
    if excepted.is_empty() {
        return check_if_no_signature_from_header();
    }

    let check_signature = check_signature(excepted);
    let missing_value = missing_value(excepted);
    quote! {
        if let Some(signature) = header.get_signature() {
            #check_signature
        } else {
            #missing_value
        }
    }
}

pub(super) fn check_if_no_signature_from_header() -> TokenStream {
    let invalid_args = invalid_args();
    quote! {
        if let Some(signature) = header.get_signature() {
            if !signature.is_empty() {
                let text = format!("too many arguments: got {}", signature);
                #invalid_args
            }
        }
    }
}

pub(super) fn create_return_msg_from_header() -> TokenStream {
    quote! {
        match header.method_return() {
            Ok(msg) => msg,
            Err(msg) => return dbus.send(msg),
        }
    }
}

pub(super) fn check_if_no_value_from_body_iter() -> TokenStream {
    let invalid_args = invalid_args();
    quote! {
        if let Some(v) = body_iter.next() {
            let mut signature = String::new();
            v.get_signature(&mut signature);
            let text = format!("Too many values: got {}", signature);
            #invalid_args
        }
    }
}

pub(super) fn unknown_interface_from_header() -> TokenStream {
    quote! {
        {
            if let Some(msg) = header.unknown_interface() {
                dbus.send(msg)?;
            }
            return Ok(());
        }
    }
}

pub(super) fn unknown_member_from_header() -> TokenStream {
    quote! {
        {
            if let Some(msg) = header.unknown_member() {
                dbus.send(msg)?;
            }
            return Ok(())
        }
    }
}

pub(super) fn unknown_property_from_header() -> TokenStream {
    quote! {
        {
            let msg = header.unknown_property(property);
            return dbus.send(msg);
        }
    }
}

pub(super) fn get_interface_from_header() -> TokenStream {
    quote! {
        if let Some(interface) = header.get_interface() {
            interface.as_ref()
        } else {
            let msg = header.error(
                std::convert::TryFrom::try_from("org.freedesktop.DBus.Error.Interface".to_string()).unwrap(),
                "Message does not have a interface".to_string(),
            );
            return dbus.send(msg);
        }
    }
}

pub(super) fn get_member_from_header() -> TokenStream {
    quote! {
        if let Some(member) = header.get_member() {
            member.as_ref()
        } else {
            let msg = header.error(
                std::convert::TryFrom::try_from("org.freedesktop.DBus.Error.Member".to_string()).unwrap(),
                "Message does not have a member".to_string(),
            );
            return dbus.send(msg);
        }
    }
}

pub(super) fn get_value_from_body_iter(
    name: &Ident,
    signature: &str,
    rust_type: &TokenStream,
    value_to_rust: &TokenStream,
) -> TokenStream {
    let missing_value = missing_value(signature);
    quote! {
        let #name: #rust_type = if let Some(i) = body_iter.next() {
            #value_to_rust
        } else {
            #missing_value
        };
    }
}

pub(super) fn get_string_from_body_iter(name: &Ident) -> TokenStream {
    let invalid_args_signature = invalid_args_signature("s");
    let rust_type = "String".parse().unwrap();
    let value_to_rust = quote! {
        {
            if let dbus_message_parser::Value::String(s) = i {
                s
            } else {
                let mut signature = String::new();
                i.get_signature(&mut signature);
                #invalid_args_signature;
            }
        }
    };
    get_value_from_body_iter(name, "s", &rust_type, &value_to_rust)
}

pub(super) fn get_variant_from_body_iter(name: &Ident) -> TokenStream {
    let invalid_args_signature = invalid_args_signature("v");
    let rust_type = "std::boxed::Box<dbus_message_parser::Value>"
        .parse()
        .unwrap();
    let value_to_rust = quote! {
        {
            if let dbus_message_parser::Value::Variant(s) = i {
                s
            } else {
                let mut signature = String::new();
                i.get_signature(&mut signature);
                #invalid_args_signature;
            }
        }
    };
    get_value_from_body_iter(name, "v", &rust_type, &value_to_rust)
}

pub(super) fn default_case_wrong_case(excepted: &str) -> TokenStream {
    let invalid_args_signature = invalid_args_signature(excepted);
    quote! {
        v => {
            let mut signature = String::new();
            v.get_signature(&mut signature);
            #invalid_args_signature
        }
    }
}

pub(super) fn create_value_to_rust(enum_type: &TokenStream, excepted: &str) -> TokenStream {
    let default_case_wrong_case = default_case_wrong_case(excepted);
    quote! {
        match i {
            dbus_message_parser::Value::#enum_type(i) => i,
            #default_case_wrong_case
        }
    }
}

pub(super) fn create_rust_to_value(enum_type: &TokenStream) -> TokenStream {
    quote! {
        dbus_message_parser::Value::#enum_type(i)
    }
}
