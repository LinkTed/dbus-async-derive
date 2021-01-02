use crate::code::{
    check_if_no_value_from_body_iter, check_result, check_signature_from_header,
    create_return_msg_from_header, get_value_from_body_iter,
};
use crate::helper::{
    get_ident_from_option_nested_meta, get_ident_from_path, get_lit_str_from_option_nested_meta,
    get_signatures_from_nested_meta,
};
use crate::introspectable::Introspectable;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::convert::TryFrom;
use syn::spanned::Spanned;
use syn::{Error as SynError, Index, LitStr, MetaList};

pub(crate) struct Method {
    name: LitStr,
    function: Ident,
    input_signatures: Vec<(String, TokenStream, TokenStream, TokenStream)>,
    output_signatures: Vec<(String, TokenStream, TokenStream, TokenStream)>,
}

impl Method {
    fn get_input_signature(&self) -> String {
        let mut result = String::new();
        for (signature, _, _, _) in &self.input_signatures {
            result += signature;
        }
        result
    }

    fn check_input_signature(&self) -> TokenStream {
        let input_signature = self.get_input_signature();
        check_signature_from_header(&input_signature)
    }

    pub(super) fn create_code(&self) -> TokenStream {
        let name = &self.name;
        let function = &self.function;

        let mut name_input_arguments = Vec::new();
        let mut parse_input_arguments = Vec::new();
        for (i, (signature, rust_type, value_to_rust, _)) in
            self.input_signatures.iter().enumerate()
        {
            let name = format_ident!("arg_{}", i.to_string());
            let parse_input_argument =
                get_value_from_body_iter(&name, signature, rust_type, value_to_rust);
            parse_input_arguments.push(parse_input_argument);
            name_input_arguments.push(name);
        }

        let mut_msg: TokenStream = if self.output_signatures.is_empty() {
            " ".parse().unwrap()
        } else {
            "mut".parse().unwrap()
        };

        let mut rust_type_output_arguments = Vec::new();
        if self.output_signatures.len() == 1 {
            let (_, rust_type, _, rust_to_value) = &self.output_signatures[0];
            rust_type_output_arguments.push(quote! {
                let i: #rust_type = result;
                let v: dbus_message_parser::Value = #rust_to_value;
                msg.add_value(v);
            });
        } else {
            for (i, (_, rust_type, _, rust_to_value)) in self.output_signatures.iter().enumerate() {
                let index = Index::from(i);
                rust_type_output_arguments.push(quote! {
                    let i: #rust_type = result.#index;
                    let v: dbus_message_parser::Value = #rust_to_value;
                    msg.add_value(v);
                });
            }
        }

        let check_input_signature = self.check_input_signature();
        let check_if_no_value_from_body_iter = check_if_no_value_from_body_iter();
        let check_result = check_result();
        let create_return_msg_from_header = create_return_msg_from_header();
        quote! {
            #name => {
                #check_input_signature
                #(#parse_input_arguments)*
                #check_if_no_value_from_body_iter;
                let result = self.#function(&dbus, &header, #(#name_input_arguments),*).await;
                let result = #check_result;
                let #mut_msg msg = #create_return_msg_from_header;
                #(#rust_type_output_arguments)*
                dbus.send(msg)?;
                return Ok(());
            }
        }
    }
}

impl Introspectable for Method {
    fn to_introspect(&self, xml: &mut String) {
        *xml += &format!("    <method name=\"{}\">\n", self.name.value());
        let mut i = 0;

        for (signature, _, _, _) in &self.input_signatures {
            let name = &format!("arg_{}", i);
            *xml += &format!(
                "      <arg type=\"{}\" name=\"{}\" direction=\"in\"/>\n",
                signature, name,
            );
            i += 1;
        }

        for (signature, _, _, _) in &self.output_signatures {
            let name = &format!("arg_{}", i);
            *xml += &format!(
                "      <arg type=\"{}\" name=\"{}\" direction=\"out\"/>\n",
                signature, name,
            );
            i += 1;
        }

        *xml += "    </method>\n";
    }
}

impl TryFrom<&MetaList> for Method {
    type Error = SynError;

    fn try_from(meta_list: &MetaList) -> Result<Self, Self::Error> {
        // Get the ident and check if it is equal "method"
        let meta_list_type = get_ident_from_path(&meta_list.path)?;
        if meta_list_type != "method" {
            return Err(SynError::new(meta_list_type.span(), "excepted \"method\""));
        }

        let nested_iter = &mut meta_list.nested.iter();

        // Get the name of the method
        let name = get_lit_str_from_option_nested_meta(nested_iter.next())?;

        // Get the name of the method to call
        let function = get_ident_from_option_nested_meta(nested_iter.next())?;

        let mut input_signature = Vec::new();
        let mut output_signature = Vec::new();
        if let Some(nested_meta) = nested_iter.next() {
            get_signatures_from_nested_meta(nested_meta, &mut input_signature)?;

            if let Some(nested_meta) = nested_iter.next() {
                get_signatures_from_nested_meta(nested_meta, &mut output_signature)?;

                if nested_iter.next().is_some() {
                    return Err(SynError::new(meta_list.span(), "too many arguments"));
                }
            }
        }

        Ok(Method {
            name,
            function,
            input_signatures: input_signature,
            output_signatures: output_signature,
        })
    }
}
