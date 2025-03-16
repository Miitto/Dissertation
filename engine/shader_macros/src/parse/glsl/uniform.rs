use proc_macro::{Diagnostic, Level, Span, TokenTree};

use crate::Result;
use crate::parse::{ident, ident_any, punct};
use crate::shader_var::ShaderType;
use crate::uniform::{LayoutBlock, SingleUniform, TextureUniform, Uniform};

use super::r#struct::parse_layout_block;
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
        check_non_block(input, info, state)
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
    state: &mut State,
) -> Result<&'a [TokenTree], Uniform, Option<Diagnostic>> {
    let (mut input, var) = parse_var(input, info)?;

    if matches!(var.t, ShaderType::Texture(_)) {
        let (input, _) = punct(';')(input).map_err(|_| {
            Diagnostic::spanned(
                input.get(0).map(|t| t.span()).unwrap_or(var.name.span()),
                Level::Error,
                "Expected ;",
            )
        })?;

        let bind = if let Some(b) = state.next_bind.take() {
            b
        } else {
            return Err(Some(Diagnostic::spanned(
                var.name.span(),
                Level::Error,
                "Expected a bind point previously",
            )));
        };

        let tex = TextureUniform { bind, var };

        return Ok((input, Uniform::Texture(tex)));
    }

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
    let (input, st) = parse_layout_block(input, info)?;

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
            st.name.span(),
            Level::Error,
            "Expected #bind directive before this line",
        )));
    };

    let uniform = Uniform::Block(LayoutBlock {
        bind,
        name: st.name,
        fields: st.fields,
        var_name,
    });

    Ok((input, uniform))
}
