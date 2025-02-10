use proc_macro2::{Delimiter, TokenTree};
use syn::{
    Token, braced,
    parse::{ParseBuffer, ParseStream},
};

use crate::{
    shader_info::ShaderInfo,
    shader_var::{ShaderFunction, ShaderStruct, ShaderVar},
};

struct State {
    pub vertex_fn_name: Option<String>,
    pub frag_fn_name: Option<String>,
    pub geometry_fn_name: Option<String>,
}

pub fn parse_glsl(input: ParseStream) -> syn::Result<ShaderInfo> {
    let mut info = ShaderInfo::default();

    let mut state = State {
        vertex_fn_name: None,
        frag_fn_name: None,
        geometry_fn_name: None,
    };

    parse_segments(input, &mut info, &mut state)?;

    Ok(info)
}

fn parse_segments(input: ParseStream, info: &mut ShaderInfo, state: &mut State) -> syn::Result<()> {
    while !input.is_empty() {
        if input.peek(Token![#]) {
            input.parse::<Token![#]>()?;
            let directive = input.parse::<syn::Ident>()?;

            match directive.to_string().as_str() {
                "vertex" => {
                    let v_name = input.parse::<syn::Ident>()?;
                    state.vertex_fn_name = Some(v_name.to_string());
                }
                "fragment" => {
                    let f_name = input.parse::<syn::Ident>()?;
                    state.frag_fn_name = Some(f_name.to_string());
                }
                "geometry" => {
                    let g_name = input.parse::<syn::Ident>()?;
                    state.geometry_fn_name = Some(g_name.to_string());
                }
                _ => {}
            }
            // TODO: Handle other preprocessor directives
            continue;
        } else if input.peek(Token![struct]) {
            let s = parse_struct(input, info)?;
            info.structs.push(s);
        } else if input.peek(syn::Ident) {
            let ident = input.parse::<syn::Ident>()?;
            match ident.to_string().as_str() {
                "uniform" => {
                    let var = parse_var(input, info)?;
                    info.uniforms.push(var);
                    input.parse::<Token![;]>()?;
                }
                _ => {
                    let name_ident = input.parse::<syn::Ident>()?;
                    let var = parse_var_pieces(ident, name_ident, info)?;
                    let params: ParseBuffer;
                    _ = syn::parenthesized!(params in input);

                    let function = parse_function(var, params, input, info)?;
                    if state
                        .vertex_fn_name
                        .as_ref()
                        .is_some_and(|n| function.name == *n)
                    {
                        info.vertex_fn = Some(function);
                    } else if state
                        .frag_fn_name
                        .as_ref()
                        .is_some_and(|n| function.name == *n)
                    {
                        info.frag_fn = Some(function);
                    } else if state
                        .geometry_fn_name
                        .as_ref()
                        .is_some_and(|n| function.name == *n)
                    {
                        info.geometry_fn = Some(function);
                    } else {
                        info.functions.push(function);
                    }
                }
            }
        } else {
            return Err(input.error("Unexpected token"));
        }
    }

    Ok(())
}

fn parse_var(segments: ParseStream, info: &ShaderInfo) -> syn::Result<ShaderVar> {
    let type_name = segments.parse::<syn::Ident>()?;
    let var_name = segments.parse::<syn::Ident>()?;

    parse_var_pieces(type_name, var_name, info)
}

fn parse_var_pieces(
    type_ident: syn::Ident,
    name_ident: syn::Ident,
    info: &ShaderInfo,
) -> syn::Result<ShaderVar> {
    let t = info.get_type(&type_ident, &[], false)?;

    Ok(ShaderVar {
        name: name_ident.to_string(),
        t,
        name_span: name_ident.span(),
        type_span: Some(type_ident.span()),
    })
}

fn parse_struct(input: ParseStream, info: &ShaderInfo) -> syn::Result<ShaderStruct> {
    input.parse::<Token![struct]>()?;
    let name = input.parse::<syn::Ident>()?;

    let field_buf: ParseBuffer;
    _ = braced!(field_buf in input);

    let mut fields = vec![];

    while !field_buf.is_empty() {
        let var = parse_var(&field_buf, info)?;
        fields.push(var);
        field_buf.parse::<Token![;]>()?;
    }

    Ok(ShaderStruct { name, fields })
}

fn parse_function(
    var: ShaderVar,
    params: ParseBuffer,
    input: ParseStream,
    info: &ShaderInfo,
) -> syn::Result<ShaderFunction> {
    let args = parse_args(params, info)?;

    let mut group = None;

    input.step(|cursor| {
        let mut rest = *cursor;
        if let Some((tt, next)) = rest.token_tree() {
            if let TokenTree::Group(g) = tt {
                if g.delimiter() == Delimiter::Brace {
                    group = Some(g);
                } else {
                    input.error("Expected a block using braces");
                }
            } else {
                input.error("Expected a block");
            }
            rest = next;
        }

        Ok(((), rest))
    })?;

    let group = group.expect("Expected a block");

    let span = group.span();
    let source = span.source_text().unwrap_or_default();

    let source = source
        .trim()
        .strip_prefix('{')
        .unwrap_or_default()
        .strip_suffix('}')
        .unwrap_or_default()
        .trim()
        .to_string();

    let pretty = source
        .lines()
        .map(|l| l.trim())
        .map(|l| {
            if l.chars().all(|c| c.is_ascii_whitespace()) {
                l.to_string()
            } else {
                format!("    {}", l)
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    let function = ShaderFunction {
        name: var.name,
        params: args,
        return_type: var.t,
        content: pretty,
    };

    Ok(function)
}

fn parse_args(input: ParseBuffer, info: &ShaderInfo) -> syn::Result<Vec<ShaderVar>> {
    let mut args = vec![];

    let mut first = true;

    while !input.is_empty() {
        if !first {
            input.parse::<Token![,]>()?;
        } else {
            first = false;
        }
        let var = parse_var(&input, info)?;
        args.push(var);
    }

    Ok(args)
}
