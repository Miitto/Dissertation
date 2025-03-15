use proc_macro::{Delimiter, Diagnostic, Level, TokenTree};

use crate::Result;
use crate::parse::{Delimited, ident, ident_any, punct};

use super::parse_var;
use crate::{
    parse::delimited,
    shader_info::ShaderInfo,
    shader_var::{ShaderStruct, ShaderVar},
};
pub fn parse_struct<'a>(
    input: &'a [TokenTree],
    info: &mut ShaderInfo,
) -> Result<&'a [TokenTree], (), Option<Diagnostic>> {
    let (input, _) = ident("struct")(input).map_err(|_| None)?;

    let (input, st) = parse_layout_block(input, info)?;

    info.structs.push(st);

    Ok((input, ()))
}

pub fn parse_layout_block<'a>(
    input: &'a [TokenTree],
    info: &ShaderInfo,
) -> Result<&'a [TokenTree], ShaderStruct, Diagnostic> {
    let (input, name) = ident_any(input)
        .map_err(|_| Diagnostic::spanned(input[0].span(), Level::Error, "Expected struct name"))?;

    let (input, Delimited { content, .. }) = delimited(Delimiter::Brace)(input)
        .map_err(|_| Diagnostic::spanned(input[0].span(), Level::Error, "Expected block"))?;

    fn struct_field<'a>(
        input: &'a [TokenTree],
        info: &ShaderInfo,
    ) -> Result<&'a [TokenTree], ShaderVar, Diagnostic> {
        let (input, var) = parse_var(input, info).map_err(|e| match e {
            Some(e) => e,
            None => Diagnostic::spanned(input[0].span(), Level::Error, "Expected Struct Field"),
        })?;

        let (input, _) = punct(';')(input)
            .map_err(|_| Diagnostic::spanned(input[0].span(), Level::Error, "Expected ;"))?;

        Ok((input, var))
    }

    let mut rest: &[TokenTree] = &content;

    let mut fields = vec![];

    while !rest.is_empty() {
        let (r, f) = struct_field(rest, info)?;
        rest = r;
        fields.push(f);
    }

    Ok((
        input,
        ShaderStruct {
            name: name.clone(),
            fields,
        },
    ))
}
