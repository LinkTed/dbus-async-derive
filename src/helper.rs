use crate::signature::SignatureIterator;
use proc_macro2::{Ident, Span, TokenStream};
use syn::spanned::Spanned;
use syn::{
    Error as SynError, Lit, LitBool, LitStr, Meta, MetaList, MetaNameValue, NestedMeta, Path,
    Result as SynResult,
};

pub(super) fn get_signatures_from_nested_meta(
    nested_meta: &NestedMeta,
    vec_signature: &mut Vec<(String, TokenStream, TokenStream, TokenStream)>,
) -> SynResult<()> {
    // Get the signature of the parameters
    let signature = get_lit_str_from_nested_meta(nested_meta)?;
    let signature_iter = SignatureIterator::from(&signature);
    for signature in signature_iter {
        vec_signature.push(signature?);
    }
    Ok(())
}

pub(super) fn get_signatures_from_option_nested_meta(
    option: Option<&NestedMeta>,
    vec_signature: &mut Vec<(String, TokenStream, TokenStream, TokenStream)>,
) -> SynResult<()> {
    match option {
        Some(nested_meta) => get_signatures_from_nested_meta(nested_meta, vec_signature),
        None => Err(SynError::new(Span::call_site(), "excepted nested_meta")),
    }
}

pub(super) fn get_meta_list_from_meta(meta: &Meta) -> SynResult<&MetaList> {
    match meta {
        Meta::List(meta_list) => Ok(meta_list),
        Meta::Path(path) => Err(SynError::new(path.span(), "excepted a MetaList got Path")),
        Meta::NameValue(name_value) => Err(SynError::new(
            name_value.span(),
            "excepted a MetaList got NameValue",
        )),
    }
}

pub(super) fn get_ident_from_path(path: &Path) -> SynResult<Ident> {
    if let Some(ident) = path.get_ident() {
        Ok(ident.clone())
    } else {
        Err(SynError::new(path.span(), "excepted \"ident\""))
    }
}

fn get_ident_from_nested_meta(nested_meta: &NestedMeta) -> SynResult<Ident> {
    match nested_meta {
        NestedMeta::Meta(Meta::Path(path)) => get_ident_from_path(path),
        NestedMeta::Meta(meta) => Err(SynError::new(meta.span(), "excepted a Path")),
        NestedMeta::Lit(lit) => Err(SynError::new(lit.span(), "excepted a Path got Literal")),
    }
}

pub(super) fn get_ident_from_option_nested_meta(option: Option<&NestedMeta>) -> SynResult<Ident> {
    match option {
        Some(nested_meta) => get_ident_from_nested_meta(nested_meta),
        None => Err(SynError::new(Span::call_site(), "excepted nested_meta")),
    }
}

pub(super) fn get_lit_str_from_lit(lit: &Lit) -> SynResult<LitStr> {
    match lit {
        Lit::Str(lit_str) => Ok(lit_str.clone()),
        lit => Err(SynError::new(lit.span(), "excepted str")),
    }
}

pub(super) fn get_lit_str_from_nested_meta(nested_meta: &NestedMeta) -> SynResult<LitStr> {
    match nested_meta {
        // It is a literal
        // Check if it is a str literal
        NestedMeta::Lit(lit) => get_lit_str_from_lit(lit),
        // It is there wrong type
        NestedMeta::Meta(meta) => Err(SynError::new(meta.span(), "excepted meta")),
    }
}

pub(super) fn get_lit_str_from_option_nested_meta(param: Option<&NestedMeta>) -> SynResult<LitStr> {
    match param {
        Some(nested_meta) => get_lit_str_from_nested_meta(nested_meta),
        // The interface have to have at least one argument as str type
        None => Err(SynError::new(
            Span::call_site(),
            "excepted at least one str argument",
        )),
    }
}

fn get_lit_bool_from_lit(lit: &Lit) -> SynResult<LitBool> {
    match lit {
        Lit::Bool(lit_bool) => Ok(lit_bool.clone()),
        lit => Err(SynError::new(lit.span(), "excepted bool")),
    }
}

fn get_lit_bool_from_nested_meta(nested_meta: &NestedMeta) -> SynResult<LitBool> {
    match nested_meta {
        NestedMeta::Lit(lit) => get_lit_bool_from_lit(lit),
        NestedMeta::Meta(meta) => Err(SynError::new(meta.span(), "excepted meta")),
    }
}

pub(super) fn get_lit_bool_from_option_nested_meta(
    param: Option<&NestedMeta>,
) -> SynResult<LitBool> {
    match param {
        Some(nested_meta) => get_lit_bool_from_nested_meta(nested_meta),
        None => Err(SynError::new(
            Span::call_site(),
            "excepted at least one bool argument",
        )),
    }
}

pub(super) fn get_meta_name_value_from_nested_meta(
    nested_meta: &NestedMeta,
) -> SynResult<&MetaNameValue> {
    match nested_meta {
        NestedMeta::Meta(Meta::NameValue(meta_name_value)) => Ok(meta_name_value),
        NestedMeta::Meta(meta) => Err(SynError::new(meta.span(), "excepted name value got meta")),
        NestedMeta::Lit(lit) => Err(SynError::new(lit.span(), "excepted name value got literal")),
    }
}

pub(super) fn get_meta_list_from_nested_meta(nested_meta: &NestedMeta) -> SynResult<&MetaList> {
    match nested_meta {
        NestedMeta::Meta(Meta::List(meta_list)) => Ok(meta_list),
        NestedMeta::Meta(meta) => Err(SynError::new(meta.span(), "excepted meta list got meta")),
        NestedMeta::Lit(lit) => Err(SynError::new(lit.span(), "excepted meta list got literal")),
    }
}
