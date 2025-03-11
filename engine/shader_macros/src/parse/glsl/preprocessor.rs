use proc_macro::{Diagnostic, Level, TokenTree};
use std::path::Path;

use crate::Result;
use crate::parse::{ident_any, punct, string, uint};

use super::{State, parse_segments};
use crate::shader_info::ShaderInfo;

pub fn parse_preprocessor<'a>(
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

            info.includes.push(path.to_string_lossy().to_string());

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
        "bind" => {
            let span = input[0].span();

            let (rest, bind) = uint(input)
                .map_err(|_| Diagnostic::spanned(span, Level::Error, "Expected bind point uint"))?;

            if bind == 0 {
                return Err(Some(Diagnostic::spanned(
                    span,
                    Level::Error,
                    "Bind point 0 is reserved for camera matrices",
                )));
            }

            if let Some(bind) = state.next_bind {
                Diagnostic::spanned(
                    span,
                    Level::Warning,
                    format!("Already have a bind for {}", bind),
                )
                .emit();
            }

            state.next_bind = Some(bind);

            input = rest;
        }

        _ => {}
    }

    Ok((input, ()))
}
