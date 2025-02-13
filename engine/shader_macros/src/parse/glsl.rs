use std::path::Path;

use proc_macro::{Delimiter, Span, TokenTree};
use proc_macro::{Diagnostic, Level};

use crate::shader_var::Uniform;
use crate::{
    Result,
    parse::string,
    shader_info::ShaderInfo,
    shader_var::{ShaderFunction, ShaderStruct, ShaderVar},
};

use super::{Delimited, delimited, ident, ident_any, punct};

#[derive(Debug, Default)]
struct State {
    pub vertex_fn_name: Option<String>,
    pub frag_fn_name: Option<String>,
    pub geometry_fn_name: Option<String>,
}

pub fn parse_glsl(input: &[TokenTree]) -> Result<(), ShaderInfo, Diagnostic> {
    let mut info = ShaderInfo::default();

    let mut state = State::default();

    parse_segments(input, &mut info, &mut state)?;

    Ok(((), info))
}

fn parse_preprocessor<'a>(
    input: &'a [TokenTree],
    info: &mut ShaderInfo,
    state: &mut State,
) -> Result<&'a [TokenTree], (), Option<Diagnostic>> {
    let (input, _) = punct('#')(input).map_err(|_| None)?;
    let (mut input, directive) = ident_any(input)
        .map_err(|_| Diagnostic::spanned(input[0].span(), Level::Error, "Expected directive"))?;

    match directive.to_string().as_str() {
        "vertex" => {
            let (rest, name) = ident_any(input).map_err(|_| {
                Diagnostic::spanned(
                    input[0].span(),
                    Level::Error,
                    "Expected vertex function name",
                )
            })?;
            if state.vertex_fn_name.is_some() {
                return Err(Some(Diagnostic::spanned(
                    input[0].span(),
                    Level::Error,
                    "Vertex function already defined",
                )));
            }
            input = rest;
            state.vertex_fn_name = Some(name.to_string());
        }
        "fragment" => {
            let (rest, name) = ident_any(input).map_err(|_| {
                Diagnostic::spanned(
                    input[0].span(),
                    Level::Error,
                    "Expected fragment function name",
                )
            })?;
            if state.frag_fn_name.is_some() {
                return Err(Some(Diagnostic::spanned(
                    input[0].span(),
                    Level::Error,
                    "Fragment function already defined",
                )));
            }
            input = rest;
            state.frag_fn_name = Some(name.to_string());
        }
        "geometry" => {
            let (rest, name) = ident_any(input).map_err(|_| {
                Diagnostic::spanned(
                    input[0].span(),
                    Level::Error,
                    "Expected geometry function name",
                )
            })?;
            if state.geometry_fn_name.is_some() {
                return Err(Some(Diagnostic::spanned(
                    input[0].span(),
                    Level::Error,
                    "Geometry function already defined",
                )));
            }
            input = rest;
            state.geometry_fn_name = Some(name.to_string());
        }
        "include" => {
            let span = input[0].span();
            let (rest, path_str) = string(input).map_err(|_| {
                Diagnostic::spanned(input[0].span(), Level::Error, "Expected include path")
            })?;
            input = rest;
            let call_file = proc_macro::Span::call_site().source_file();
            let call_path = call_file.path();
            let mut parent = if let Some(parent) = call_path.parent() {
                parent
            } else {
                return Err(Some(Diagnostic::spanned(
                    input[0].span(),
                    Level::Error,
                    "Failed to get parent path",
                )));
            };

            let mut path = Path::new(&path_str).to_path_buf();
            if path_str.chars().next().is_some_and(|c| c == '.') {
                path = parent.join(path);
            } else {
                // Path goes to "" before becoming none, need to check two levels up
                while parent.parent().is_some_and(|p| p.parent().is_some()) {
                    parent = parent.parent().unwrap();
                }
                path = std::path::absolute(parent.join(path)).expect("Failed to get absolute path");
            }
            let include_input = std::fs::read_to_string(path).expect("Failed to read file");
            let token_stream2: proc_macro2::TokenStream =
                syn::parse_str(&include_input).expect("Failed to parse included file");
            let token_stream: proc_macro::TokenStream = token_stream2.into();
            let mut collected: Vec<TokenTree> = token_stream.into_iter().collect();
            for token in &mut collected {
                token.set_span(span);
            }
            parse_segments(&collected, info, state)?;
        }

        _ => {}
    }

    Ok((input, ()))
}

fn parse_struct<'a>(
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

fn parse_segments(
    mut input: &[TokenTree],
    info: &mut ShaderInfo,
    state: &mut State,
) -> Result<(), (), Diagnostic> {
    while !input.is_empty() {
        match parse_preprocessor(input, info, state) {
            Ok((rest, _)) => {
                input = rest;
                continue;
            }
            Err(diag) => {
                if let Some(diag) = diag {
                    return Err(diag);
                }
            }
        }
        match parse_struct(input, info) {
            Ok((rest, _)) => {
                input = rest;
                continue;
            }
            Err(diag) => {
                if let Some(diag) = diag {
                    return Err(diag);
                }
            }
        }
        match parse_uniform(input, info) {
            Ok((rest, _)) => {
                input = rest;
                continue;
            }
            Err(diag) => {
                if let Some(diag) = diag {
                    return Err(diag);
                }
            }
        }
        match parse_function(input, info) {
            Ok((rest, function)) => {
                input = rest;

                let fn_name = function.var.name.to_string();

                if state
                    .vertex_fn_name
                    .as_ref()
                    .is_some_and(|name| *name == fn_name)
                {
                    info.vertex_fn = Some(function);
                } else if state
                    .frag_fn_name
                    .as_ref()
                    .is_some_and(|name| *name == fn_name)
                {
                    info.frag_fn = Some(function);
                } else if state
                    .geometry_fn_name
                    .as_ref()
                    .is_some_and(|name| *name == fn_name)
                {
                    info.geometry_fn = Some(function);
                } else {
                    info.functions.push(function);
                }
                continue;
            }
            Err(diag) => {
                if let Some(diag) = diag {
                    return Err(diag);
                }
            }
        }

        return Err(Diagnostic::spanned(
            input.first().map(|i| i.span()).unwrap_or(Span::call_site()),
            Level::Error,
            "Expected preprocessor directive, variable, struct, or function",
        ));
    }

    Ok(((), ()))
}

fn parse_uniform<'a>(
    input: &'a [TokenTree],
    info: &mut ShaderInfo,
) -> Result<&'a [TokenTree], (), Option<Diagnostic>> {
    let (input, _) = ident("uniform")(input).map_err(|_| None)?;

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

    let uniform = Uniform { var, value };

    info.uniforms.push(uniform);

    Ok((input, ()))
}

fn parse_var<'a>(
    input: &'a [TokenTree],
    info: &ShaderInfo,
) -> Result<&'a [TokenTree], ShaderVar, Option<Diagnostic>> {
    if input.is_empty() {
        return Err(None);
    }

    let (input, type_name) = ident_any(input).map_err(|_| {
        Diagnostic::spanned(
            input[0].span(),
            Level::Error,
            format!("Expected type, got {}", input[0]),
        )
    })?;
    let (input, name) = ident_any(input)
        .map_err(|_| Diagnostic::spanned(input[0].span(), Level::Error, "Expected field name"))?;

    if let Ok(type_found) = info.get_type(type_name, &[], false) {
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

fn parse_function<'a>(
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
                Diagnostic::spanned(params_content[0].span(), Level::Error, "Expected ,")
            })?;
            params_content = rest;
        } else {
            first = false;
        }

        let (rest, var) = parse_var(params_content, info)?;
        args.push(var);
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
        .map(|t| t.to_string())
        .collect::<Vec<_>>()
        .join(" ");

    let function = ShaderFunction {
        var,
        params: args,
        content: stringified,
    };

    Ok((input, function))
}
