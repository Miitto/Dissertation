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

            if !call_file.is_real() {
                return Err(Some(Diagnostic::spanned(
                    input[0].span(),
                    Level::Error,
                    "Can't include a fake file",
                )));
            }
            let mut path = Path::new(&path_str).to_path_buf();

            let root = std::env::current_dir().expect("Failed to get current dir");

            let mut set_path = false;

            let root_join = root.join(&path);
            if root_join.exists() {
                path = root_join;
                set_path = true;
            }

            if !set_path {
                let root_src = root.join("src");
                let src_join = root_src.join(&path);
                if src_join.exists() {
                    path = src_join;
                    set_path = true;
                }
            }

            if !set_path {
                let call_path = std::path::absolute(call_file.path()).map_err(|e| {
                    Some(Diagnostic::spanned(
                        input[0].span(),
                        Level::Error,
                        format!("Can't get absolute path: {:?}", e),
                    ))
                })?;

                if path_str.chars().next().is_some_and(|c| c == '.') {
                    path = call_path
                        .parent()
                        .expect("Relative path has no parent")
                        .join(path);
                } else {
                    let mut parent = call_path.parent();
                    while let Some(par) = parent {
                        let p = par.join(&path);
                        if p.exists() {
                            path = p;
                            break;
                        }
                        parent = par.parent();
                    }
                }
            }

            if !path.exists() {
                return Err(Some(Diagnostic::spanned(
                    input[0].span(),
                    Level::Error,
                    "Can't find file",
                )));
            }

            info.includes.push(path.to_string_lossy().to_string());

            let include_input = std::fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Failed to read file: {}", path.to_string_lossy()));
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
        "snippet" => {
            let mut namespaces = vec![];

            if ident_any(input).is_err() {
                return Err(Some(Diagnostic::spanned(
                    directive.span(),
                    Level::Error,
                    "Expected Include Path",
                )));
            }

            while let Ok((rest, ident)) = ident_any(input) {
                namespaces.push(ident.clone());

                input = rest;

                let (rest, p) = if let Ok(res) = punct(':')(rest) {
                    res
                } else {
                    break;
                };

                let (rest, _) = punct(':')(rest)
                    .map_err(|_| Diagnostic::spanned(p.span(), Level::Error, "Expected ::"))?;

                input = rest;
            }

            // Optional trailing semicolon
            if let Ok((rest, _)) = punct(';')(input) {
                input = rest;
            }

            info.uses.push(namespaces);
        }

        _ => {}
    }

    Ok((input, ()))
}
