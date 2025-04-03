use proc_macro::{Delimiter, Diagnostic, Level, Span, TokenTree};

use crate::Result;
use crate::parse::{Delimited, ident, punct};
use crate::shader_var::{InputQualifier, ShaderFunctionParam};

use super::parse_var;
use crate::{parse::delimited, shader_info::ShaderInfo, shader_var::ShaderFunction};

pub fn parse_function<'a>(
    input: &'a [TokenTree],
    info: &ShaderInfo,
) -> Result<&'a [TokenTree], ShaderFunction, Option<Diagnostic>> {
    let (input, var) = parse_var(input, info)?;

    let (input, params) = delimited(Delimiter::Parenthesis)(input).map_err(|_| {
        Diagnostic::spanned(
            input.first().map(|i| i.span()).unwrap_or(Span::call_site()),
            Level::Error,
            "Expected function parameters",
        )
    })?;

    let mut params_content: &[TokenTree] = &params.content;

    let mut args = vec![];

    let mut first = true;
    while !params_content.is_empty() {
        if !first {
            let (rest, _) = punct(',')(params_content).map_err(|_| {
                Diagnostic::spanned(
                    params_content
                        .first()
                        .expect("Content should of had at least one element")
                        .span(),
                    Level::Error,
                    "Expected ,",
                )
            })?;
            params_content = rest;
        } else {
            first = false;
        }

        let qual = if let Ok((rest, _)) = ident("in")(params_content) {
            params_content = rest;
            Some(InputQualifier::In)
        } else if let Ok((rest, _)) = ident("out")(params_content) {
            params_content = rest;
            Some(InputQualifier::Out)
        } else if let Ok((rest, _)) = ident("inout")(params_content) {
            params_content = rest;
            Some(InputQualifier::InOut)
        } else {
            None
        };

        let (rest, var) = parse_var(params_content, info)?;

        let param = ShaderFunctionParam {
            var,
            qualifier: qual,
        };

        args.push(param);
        params_content = rest;
    }

    let (input, Delimited { content, .. }) = delimited(Delimiter::Brace)(input).map_err(|_| {
        Diagnostic::spanned(
            input.first().map(|i| i.span()).unwrap_or(Span::call_site()),
            Level::Error,
            "Expected function body",
        )
    })?;

    let stringified = content
        .into_iter()
        .map(|t| {
            if let TokenTree::Punct(p) = &t {
                if p.as_char() == ';' {
                    return ";\n".to_string();
                }
            }
            t.to_string()
        })
        .collect::<Vec<_>>()
        .join(" ");

    let function = ShaderFunction {
        var,
        params: args,
        content: stringified,
    };

    Ok((input, function))
}
