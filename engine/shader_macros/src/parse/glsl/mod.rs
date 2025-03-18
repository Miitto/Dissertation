use proc_macro::{Diagnostic, Level, Span, TokenTree};

use crate::{
    Result,
    shader_info::{ComputeInfo, ShaderInfo},
};

mod buffer;
mod function;
mod preprocessor;
mod r#struct;
mod uniform;
mod variable;

use buffer::parse_buffer;
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
    pub next_size: Option<(u32, u32, u32)>,
    pub kernel_names: Vec<String>,
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
        match parse_buffer(input, info, state) {
            Ok((rest, buf)) => {
                input = rest;
                info.buffers.push(buf);
                continue;
            }
            Err(e) => {
                if let Some(e) = e {
                    return Err(e);
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
                } else if state.kernel_names.contains(&fn_name) {
                    let size = state.next_size.take().unwrap_or((1, 1, 1));

                    info.compute.push(ComputeInfo {
                        name: function.var.name.clone(),
                        size,
                        function,
                    });
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
