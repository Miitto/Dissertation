use proc_macro::{Diagnostic, Level, TokenTree};

use crate::{
    ShaderInfo,
    parse::{glsl::variable::parse_var, ident, punct},
    shader_var::{ShaderConstant, ShaderVar},
};

pub fn parse_constant<'a>(
    input: &'a [TokenTree],
    info: &mut ShaderInfo,
) -> Result<(&'a [TokenTree], ShaderConstant), Option<Diagnostic>> {
    let (input, _) = ident("const")(input).map_err(|_| None)?;

    let (input, var) = parse_var(input, info)?;

    let (mut input, _) = punct('=')(input).map_err(|_| {
        if input.is_empty() {
            Diagnostic::new(Level::Error, "Unexpected end of input, expected =")
        } else {
            Diagnostic::spanned(input[0].span(), Level::Error, "Expected =")
        }
    })?;

    let mut tokens = vec![];

    while !input.is_empty() {
        if let Ok((_, _)) = punct(';')(input) {
            input = &input[1..];
            break;
        } else {
            tokens.push(input[0].clone());
            input = &input[1..];
        }
    }

    let value = tokens
        .iter()
        .map(|t| t.to_string())
        .collect::<Vec<String>>()
        .join(" ");

    Ok((input, ShaderConstant { var, value }))
}
