use proc_macro::{Delimiter, Diagnostic, Level, Span, TokenTree};

use crate::Result;
use crate::parse::{Delimited, delimited, ident, ident_any, punct};
use crate::shader_var::ShaderVar;
use crate::uniform::{SingleUniform, Uniform, UniformBlock};

use super::{State, parse_var};
use crate::shader_info::ShaderInfo;

pub fn parse_uniform<'a>(
    input: &'a [TokenTree],
    info: &mut ShaderInfo,
    state: &mut State,
) -> Result<&'a [TokenTree], (), Option<Diagnostic>> {
    let (input, uniform_ident) = ident("uniform")(input).map_err(|_| None)?;

    if input.is_empty() {
        return Err(Some(Diagnostic::spanned(
            uniform_ident.span(),
            Level::Error,
            "Expected uniform declaration",
        )));
    }

    ident_any(input).map_err(|_| {
        Some(Diagnostic::spanned(
            input[0].span(),
            Level::Error,
            "Expected type or block name",
        ))
    })?;

    let (input, uniform) = if is_block(input) {
        check_block(input, info, state)
    } else {
        check_non_block(input, info)
    }?;

    info.uniforms.push(uniform);

    Ok((input, ()))
}

fn is_block(input: &[TokenTree]) -> bool {
    input
        .get(1)
        .filter(|t| matches!(t, TokenTree::Group(_)))
        .is_some()
}

fn check_non_block<'a>(
    input: &'a [TokenTree],
    info: &mut ShaderInfo,
) -> Result<&'a [TokenTree], Uniform, Option<Diagnostic>> {
    let (mut input, var) = parse_var(input, info)?;

    let mut value = None;

    if let Ok((mut rest, _)) = punct('=')(input) {
        let mut sources = vec![];
        while punct(';')(rest).is_err() {
            sources.push(rest[0].to_string());

            rest = &rest[1..];
        }

        value = Some(sources.join(" "));
        input = &rest[1..];
    } else {
        let (rest, _) = punct(';')(input).map_err(|_| {
            Diagnostic::spanned(
                input.first().map(|i| i.span()).unwrap_or(Span::call_site()),
                Level::Error,
                "Expected ; after uniform declaration",
            )
        })?;
        input = rest;
    }

    let uniform = Uniform::Single(SingleUniform { var, value });

    Ok((input, uniform))
}

fn check_block<'a>(
    input: &'a [TokenTree],
    info: &mut ShaderInfo,
    state: &mut State,
) -> Result<&'a [TokenTree], Uniform, Option<Diagnostic>> {
    let (input, block_name) = ident_any(input).map_err(|_| {
        Some(Diagnostic::spanned(
            input[0].span(),
            Level::Error,
            "Expected name for uniform block",
        ))
    })?;

    let (input, Delimited { content, .. }) = delimited(Delimiter::Brace)(input).map_err(|_| {
        Some(Diagnostic::spanned(
            input.first().map(|i| i.span()).unwrap_or(Span::call_site()),
            Level::Error,
            "Expected uniform block content",
        ))
    })?;

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

    let (input, var_name) = ident_any(input)
        .map(|(i, v)| (i, Some(v.clone())))
        .unwrap_or((input, None));

    let (input, _) = punct(';')(input).map_err(|_| {
        if input.is_empty() {
            Diagnostic::new(Level::Error, "Unexpected end of input, expected ;")
        } else {
            Diagnostic::spanned(input[0].span(), Level::Error, "Expected ;")
        }
    })?;

    let bind = if let Some(bind) = state.next_bind.take() {
        bind
    } else {
        return Err(Some(Diagnostic::spanned(
            block_name.span(),
            Level::Error,
            "Expected #bind directive before this line",
        )));
    };

    let uniform = Uniform::Block(UniformBlock {
        bind,
        name: block_name.clone(),
        fields,
        var_name,
    });

    Ok((input, uniform))
}
