use proc_macro::{Delimiter, Diagnostic, Level, Span, TokenTree};

use crate::Result;
use crate::parse::{Delimited, ident, ident_any, punct};

use super::parse_var;
use crate::{
    parse::delimited,
    shader_info::ShaderInfo,
    shader_var::{ShaderFunction, ShaderStruct, ShaderVar},
};
pub fn parse_struct<'a>(
    input: &'a [TokenTree],
    info: &mut ShaderInfo,
) -> Result<&'a [TokenTree], (), Option<Diagnostic>> {
    let (input, _) = ident("struct")(input).map_err(|_| None)?;
    let (input, name) = ident_any(input)
        .map_err(|_| Diagnostic::spanned(input[0].span(), Level::Error, "Expected struct name"))?;

    let (input, Delimited { content, .. }) = delimited(Delimiter::Brace)(input)
        .map_err(|_| Diagnostic::spanned(input[0].span(), Level::Error, "Expected block"))?;

    fn struct_field<'a>(
        input: &'a [TokenTree],
        info: &mut ShaderInfo,
    ) -> Result<&'a [TokenTree], ShaderVar, Option<Diagnostic>> {
        let (input, var) = parse_var(input, info)?;

        let (input, _) = punct(';')(input)
            .map_err(|_| Diagnostic::spanned(input[0].span(), Level::Error, "Expected ;"))?;

        Ok((input, var))
    }

    let mut rest: &[TokenTree] = &content;

    let mut fields = vec![];

    while !content.is_empty() {
        match struct_field(rest, info) {
            Ok((r, f)) => {
                fields.push(f);
                rest = r;
            }
            Err(diag) => {
                if let Some(diag) = diag {
                    return Err(Some(diag));
                } else {
                    break;
                }
            }
        }
    }

    info.structs.push(ShaderStruct {
        name: name.clone(),
        fields,
    });

    Ok((input, ()))
}
