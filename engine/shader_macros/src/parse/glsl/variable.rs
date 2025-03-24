use proc_macro::{Diagnostic, Ident, Level, Span, TokenTree};

use crate::Result;
use crate::parse::{delimited, ident_any};
use crate::shader_var::ShaderStruct;

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

    let (input, is_array) = if let Ok((i, _)) = delimited(proc_macro::Delimiter::Bracket)(input) {
        (i, true)
    } else {
        (input, false)
    };

    if let Ok(type_found) = info.get_type(type_name) {
        let var = ShaderVar {
            name: name.clone(),
            t: type_found,
            type_span: Some(type_name.span()),
            is_array,
        };

        Ok((input, var))
    } else {
        let var = ShaderVar {
            name: name.clone(),
            t: crate::shader_var::ShaderType::Struct(ShaderStruct {
                name: type_name.clone(),
                fields: vec![],
            }),
            type_span: None,
            is_array: false,
        };

        Ok((input, var))
    }
}
