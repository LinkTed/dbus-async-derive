use crate::code::{
    check_if_no_signature_from_header, check_if_no_value_from_body_iter,
    create_return_msg_from_header, get_member_from_header, unknown_member_from_header,
};
use crate::helper::{get_ident_from_path, get_lit_bool_from_option_nested_meta};
use crate::interface::Interface;
use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Error as SynError, MetaList, Result as SynResult};

static START_XML: &str = r#"
<!DOCTYPE node PUBLIC "-//freedesktop//DTD D-BUS Object Introspection 1.0//EN"
                      "http://www.freedesktop.org/standards/dbus/1.0/introspect.dtd">
<node>
   <interface name="org.freedesktop.DBus.Introspectable">
     <method name="Introspect">
       <arg type="s" name="xml_data" direction="out"/>
     </method>
   </interface>
"#;

static PROPERTIES_XML: &str = r#"
   <interface name="org.freedesktop.DBus.Properties">
     <method name="Get">
       <arg type="s" name="interface_name" direction="in"/>
       <arg type="s" name="property_name" direction="in"/>
       <arg type="v" name="value" direction="out"/>
     </method>
     <method name="GetAll">
       <arg type="s" name="interface_name" direction="in"/>
       <arg type="a{sv}" name="properties" direction="out"/>
     </method>
     <method name="Set">
       <arg type="s" name="interface_name" direction="in"/>
       <arg type="s" name="property_name" direction="in"/>
       <arg type="v" name="value" direction="in"/>
     </method>
     <signal name="PropertiesChanged">
       <arg type="s" name="interface_name"/>
       <arg type="a{sv}" name="changed_properties"/>
       <arg type="as" name="invalidated_properties"/>
     </signal>
   </interface>
"#;

pub(super) trait Introspectable {
    fn to_introspect(&self, xml: &mut String);
}

pub(super) fn create_introspectable_code(
    interfaces: &[Interface],
    have_properties: bool,
) -> TokenStream {
    let mut xml = START_XML.to_string();

    if have_properties {
        xml += PROPERTIES_XML;
    }

    for interface in interfaces {
        interface.to_introspect(&mut xml);
    }

    let check_if_no_signature_from_header = check_if_no_signature_from_header();
    let check_if_no_value_from_body_iter = check_if_no_value_from_body_iter();
    let get_member_from_header = get_member_from_header();
    let create_return_msg_from_header = create_return_msg_from_header();
    let unknown_member_from_header = unknown_member_from_header();
    quote! {
        "org.freedesktop.DBus.Introspectable" => {
             match #get_member_from_header {
                 "Introspect" => {
                     #check_if_no_signature_from_header;
                     #check_if_no_value_from_body_iter;
                     let mut xml = #xml.to_string();
                     if let Some(path) = header.get_path() {
                        let list = dbus.list_method_call(path.clone()).await?;
                        for l in list {
                            xml += &format!("  <node name=\"{}\"/>\n", l);
                        }
                     }
                     xml += "</node>";
                     let mut msg = #create_return_msg_from_header;
                     msg.add_value(dbus_message_parser::Value::String(xml));
                     return dbus.send(msg);
                 }
                 _ => #unknown_member_from_header
             }
        }
    }
}

pub(super) fn parse_introspectable(meta_list: &MetaList) -> SynResult<bool> {
    let meta_list_type = get_ident_from_path(&meta_list.path)?;
    if meta_list_type != "introspectable" {
        return Err(SynError::new(
            meta_list_type.span(),
            "excepted \"introspectable\"",
        ));
    }
    let nested_iter = &mut meta_list.nested.iter();

    let lit_bool = get_lit_bool_from_option_nested_meta(nested_iter.next())?;

    if let Some(_) = nested_iter.next() {
        return Err(SynError::new(meta_list.span(), "too many arguments"));
    }
    Ok(lit_bool.value)
}
