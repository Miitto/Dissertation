use proc_macro::{Delimiter, Diagnostic, Level, Span, TokenTree};

use crate::Result;
use crate::parse::{Delimited, ident_any, punct};

use crate::{shader_info::ShaderInfo, shader_var::ShaderVar};

pub fn parse_var<'a>(
    input: &'a [TokenTree],
    info: &ShaderInfo,
) -> Result<&'a [TokenTree], ShaderVar, Option<Diagnostic>> {
    if input.is_empty() {
        return Err(None);
    }

    let (input, type_name) = ident_any(input).map_err(|_| {
        Diagnostic::spanned(
            input
                .first()
                .expect("Input should of had at least one element")
                .span(),
            Level::Error,
            format!("Expected type, got {}", input[0]),
        )
    })?;
    let (input, name) = ident_any(input).map_err(|_| {
        Diagnostic::spanned(
            input
                .first()
                .expect("Input should of at least had one element")
                .span(),
            Level::Error,
            "Expected field name",
        )
    })?;

    if let Ok(type_found) = info.get_type(type_name) {
        let var = ShaderVar {
            name: name.clone(),
            t: type_found,
            type_span: Some(type_name.span()),
        };

        Ok((input, var))
    } else {
        Err(Some(Diagnostic::spanned(
            type_name.span(),
            Level::Error,
            "Unknown type",
        )))
    }
}
