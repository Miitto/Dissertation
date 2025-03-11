use proc_macro::{Diagnostic, Level, Span, TokenTree};

use crate::{Result, shader_info::ShaderInfo};

mod function;
mod preprocessor;
mod r#struct;
mod uniform;
mod variable;

use function::parse_function;
use preprocessor::parse_preprocessor;
use r#struct::parse_struct;
use uniform::parse_uniform;
use variable::parse_var;

#[derive(Debug, Default)]
struct State {
    pub vertex_fn_name: Option<String>,
    pub frag_fn_name: Option<String>,
    pub geometry_fn_name: Option<String>,
    pub next_bind: Option<u32>,
}

pub fn parse_glsl(input: &[TokenTree]) -> Result<(), ShaderInfo, Diagnostic> {
    let mut info = ShaderInfo::default();

    let mut state = State::default();

    parse_segments(input, &mut info, &mut state)?;

    Ok(((), info))
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
        match parse_uniform(input, info, state) {
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
