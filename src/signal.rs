use crate::helper::{
    get_ident_from_path, get_lit_str_from_option_nested_meta,
    get_signatures_from_option_nested_meta,
};
use crate::introspectable::Introspectable;
use proc_macro2::TokenStream;
use std::convert::TryFrom;
use syn::spanned::Spanned;
use syn::{Error as SynError, LitStr, MetaList};

pub(crate) struct Signal {
    name: LitStr,
    signatures: Vec<(String, TokenStream, TokenStream, TokenStream)>,
}

impl Introspectable for Signal {
    fn to_introspect(&self, xml: &mut String) {
        *xml += &format!("    <signal name=\"{}\">\n", self.name.value());
        for (signature, _, _, _) in &self.signatures {
            *xml += &format!("      <arg type=\"{}\"/>\n", signature);
        }
        *xml += "    </signal>\n";
    }
}

impl TryFrom<&MetaList> for Signal {
    type Error = SynError;

    fn try_from(meta_list: &MetaList) -> Result<Self, Self::Error> {
        // Get the ident and check if it is equal "signal"
        let meta_list_type = get_ident_from_path(&meta_list.path)?;
        if meta_list_type != "signal" {
            return Err(SynError::new(meta_list_type.span(), "excepted \"signal\""));
        }

        let nested_iter = &mut meta_list.nested.iter();

        // Get the name of the signal
        let name = get_lit_str_from_option_nested_meta(nested_iter.next())?;

        let mut signatures = Vec::new();
        get_signatures_from_option_nested_meta(nested_iter.next(), &mut signatures)?;

        if let Some(_) = nested_iter.next() {
            return Err(SynError::new(meta_list.span(), "too many arguments"));
        }
        Ok(Signal { name, signatures })
    }
}
