use crate::code::{
    check_signature, create_rust_to_value, create_value_to_rust, default_case_wrong_case,
    missing_value,
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Error as SynError, LitStr, Result as SynResult};

/// Iterator for signature.
/// This Iterator returns a single signature type and with the corresponding
/// `dbus_message_parser::Value` type.
pub struct SignatureIterator {
    span: Span,
    signature: String,
    offset: usize,
}

impl From<&LitStr> for SignatureIterator {
    fn from(signature: &LitStr) -> Self {
        SignatureIterator {
            span: signature.span(),
            signature: signature.value(),
            offset: 0,
        }
    }
}

impl SignatureIterator {
    fn basic(
        &mut self,
        value_type: &str,
        rust_type: &str,
        signature: &'static str,
    ) -> SynResult<Option<(&str, TokenStream, TokenStream, TokenStream)>> {
        self.offset += 1;
        let value_type = value_type.parse().unwrap();
        let rust_to_value = create_rust_to_value(&value_type);
        let rust_type = rust_type.parse().unwrap();
        let value_to_rust = create_value_to_rust(&value_type, signature);
        Ok(Some((signature, rust_type, value_to_rust, rust_to_value)))
    }

    /// Get the next single signature.
    fn get_next(&mut self) -> SynResult<Option<(&str, TokenStream, TokenStream, TokenStream)>> {
        // Get the next character
        let s = if let Some(s) = self.signature.get(self.offset..(self.offset + 1)) {
            if let Some(s) = s.chars().next() {
                s
            } else {
                return Ok(None);
            }
        } else {
            return Ok(None);
        };

        match s {
            'y' => self.basic("Byte", "u8", "y"),
            'b' => self.basic("Boolean", "bool", "b"),
            'n' => self.basic("Int16", "i16", "n"),
            'q' => self.basic("Uint16", "u16", "q"),
            'i' => self.basic("Int32", "i32", "i"),
            'u' => self.basic("Uint32", "u32", "u"),
            'x' => self.basic("Int64", "i64", "x"),
            't' => self.basic("Uint64", "u64", "t"),
            's' => self.basic("String", "String", "s"),
            'o' => self.basic("ObjectPath", "String", "o"),
            'g' => self.basic("Signature", "String", "g"),
            'v' => self.basic("Variant", "std::boxed::Box<Value>", "v"),
            'a' => {
                let start_offset = self.offset;
                // It is an array
                self.offset += 1;
                // Get the type of the array
                if let Some(next) = self.get_next()? {
                    let (
                        inner_signature,
                        inner_rust_type,
                        inner_value_to_rust,
                        inner_rust_to_value,
                    ) = next;
                    let inner_signature = inner_signature.to_string();
                    let signature = &self.signature[start_offset..self.offset];
                    let check_signature = check_signature(&inner_signature);
                    let default_case_wrong_case = default_case_wrong_case(signature);
                    let rust_type = quote! { Vec<#inner_rust_type>};
                    let value_to_rust = quote! {
                        match i {
                            dbus_message_parser::Value::Array(i, signature) => {
                                #check_signature;
                                let mut o = Vec::new();
                                for i in i {
                                    o.push(#inner_value_to_rust);
                                }
                                o
                            }
                            #default_case_wrong_case
                        }
                    };
                    let rust_to_value = quote! {
                        {
                            let mut o = Vec::new();
                            for i in i {
                                o.push(#inner_rust_to_value)
                            }
                            dbus_message_parser::Value::Array(o, #inner_signature.to_string())
                        }
                    };

                    return Ok(Some((signature, rust_type, value_to_rust, rust_to_value)));
                } else {
                    return Err(SynError::new(self.span, "Array was the last character"));
                }
            }
            '(' => {
                let start_offset = self.offset;
                self.offset += 1;
                let mut rust_type = "(".to_string();
                let mut vec_inner_value_to_rust = Vec::new();
                let mut vec_inner_value_to_rust_return = Vec::new();
                let mut vec_inner_rust_to_value = Vec::new();
                loop {
                    if let Some(s) = self.signature.get(self.offset..(self.offset + 1)) {
                        if s == ")" {
                            self.offset += 1;
                            let signature = &self.signature[start_offset..self.offset];
                            rust_type += ")";
                            if rust_type == "()" {
                                return Err(SynError::new(self.span, "struct is empty"));
                            } else {
                                let default_case_wrong_case = default_case_wrong_case(signature);
                                let rust_type = rust_type.parse().unwrap();
                                let value_to_rust = quote! {
                                    match i {
                                        dbus_message_parser::Value::Struct(i) => {
                                            let mut i_iter = i.into_iter();
                                            #(#vec_inner_value_to_rust)*
                                            (#(#vec_inner_value_to_rust_return),*)
                                        }
                                        #default_case_wrong_case
                                    }
                                };
                                let rust_to_value = quote! {
                                    {
                                        let i_tuple = i;
                                        let mut o = Vec::new();
                                        #(#vec_inner_rust_to_value)*
                                        o
                                    }
                                };
                                return Ok(Some((
                                    signature,
                                    rust_type,
                                    value_to_rust,
                                    rust_to_value,
                                )));
                            }
                        } else if let Some(next) = self.get_next()? {
                            let (
                                inner_signature,
                                inner_rust_type,
                                inner_value_to_rust,
                                inner_rust_to_value,
                            ) = next;
                            let inner_signature = inner_signature.to_string();
                            rust_type += &inner_rust_type.to_string();
                            rust_type += ", ";
                            let o = Ident::new(
                                &format!("o{}", vec_inner_value_to_rust.len()),
                                self.span,
                            );
                            let missing_value = missing_value(&inner_signature);
                            let inner_rust_type_conv = quote! {
                                let #o = if let Some(i) = i_iter.next() {
                                    #inner_value_to_rust
                                } else {
                                    #missing_value
                                };
                            };
                            vec_inner_value_to_rust.push(inner_rust_type_conv);
                            vec_inner_value_to_rust_return.push(o);

                            let i = vec_inner_rust_to_value.len();
                            let inner_enum_type_conv = quote! {
                                let i = i_tuple.#i;
                                o.push(#inner_rust_to_value);
                            };
                            vec_inner_rust_to_value.push(inner_enum_type_conv);
                        } else {
                            return Err(SynError::new(self.span, ") was not closed"));
                        }
                    } else {
                        return Err(SynError::new(self.span, ") was not closed"));
                    }
                }
            }
            '{' => {
                let start_offset = self.offset;
                self.offset += 1;
                if let Some(next) = self.get_next()? {
                    let (_, key_rust_type, key_value_to_rust, key_rust_to_value) = next;
                    if let Some(next) = self.get_next()? {
                        let (_, value_rust_type, value_value_to_rust, value_rust_to_value) = next;
                        let rust_type = format!("({}, {})", key_rust_type, value_rust_type);
                        if let Some(s) = self.signature.get(self.offset..(self.offset + 1)) {
                            if s == "}" {
                                self.offset += 1;
                                let signature = &self.signature[start_offset..self.offset];
                                let default_case_wrong_case = default_case_wrong_case(signature);
                                let rust_type = rust_type.parse().unwrap();
                                let value_to_rust = quote! {
                                    match i {
                                        dbus_message_parser::Value::DictEntry(i_entry) => {
                                            let i_entry = *i_entry;

                                            let i = i_entry.0;
                                            let key = #key_value_to_rust;

                                            let i = i_entry.1;
                                            let value = #value_value_to_rust;

                                            (key, value)
                                        }
                                        #default_case_wrong_case
                                    }
                                };
                                let rust_to_value = quote! {
                                    {
                                        let entry = i;

                                        let i = entry.0;
                                        let key = #key_rust_to_value;

                                        let i = entry.1;
                                        let value = #value_rust_to_value;

                                        dbus_message_parser::Value::DictEntry(Box::new((key, value)))
                                    }
                                };
                                return Ok(Some((
                                    signature,
                                    rust_type,
                                    value_to_rust,
                                    rust_to_value,
                                )));
                            } else {
                                return Err(SynError::new(self.span, "} was not closed"));
                            }
                        } else {
                            return Err(SynError::new(self.span, "} was not closed"));
                        }
                    } else {
                        return Err(SynError::new(self.span, "Could not get value type"));
                    }
                } else {
                    return Err(SynError::new(self.span, "Could not get key type"));
                }
            }
            unknown_char => Err(SynError::new(
                self.span,
                format!("unknown signature: {}", unknown_char),
            )),
        }
    }
}

impl Iterator for SignatureIterator {
    type Item = SynResult<(String, TokenStream, TokenStream, TokenStream)>;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the next signature
        match self.get_next() {
            Ok(r) => match r {
                Some((signature, rust_type, value_to_rust, rust_to_value)) => Some(Ok((
                    signature.to_string(),
                    rust_type,
                    value_to_rust,
                    rust_to_value,
                ))),
                None => None,
            },
            Err(e) => Some(Err(e)),
        }
    }
}
