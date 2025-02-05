use proc_macro::{Delimiter, Group, Punct, Spacing, Span, TokenTree};

use crate::{
    errors::ShaderError,
    shader_info::ShaderInfo,
    shader_var::{ShaderFunction, ShaderStruct, ShaderVar},
    type_checking,
};

#[derive(Clone, Debug)]
enum Segment {
    Line(Vec<TokenTree>),
    Group(Vec<TokenTree>, Block),
}

#[derive(Clone, Debug)]
struct Block {
    span: Span,
    segments: Vec<Segment>,
}

pub fn parse_glsl(input: Group) -> ShaderInfo {
    let segments = build_segments(input);

    let mut info = ShaderInfo::default();

    parse_segments(&segments, &mut info);

    info
}

fn parse_segments(segments: &[Segment], info: &mut ShaderInfo) {
    let mut iterator = segments.iter().peekable();

    let mut vertex_fn_name = None;
    let mut frag_fn_name = None;
    let mut geometry_fn_name = None;

    while let Some(next) = iterator.next().as_ref() {
        match next {
            Segment::Line(l) => {
                let first = l
                    .first()
                    .expect("Segment has no first element but is not empty");

                match first {
                    TokenTree::Punct(p) => {
                        if p.as_char() == '#' {
                            let directive = if let Some(dir) = l.get(1) {
                                dir
                            } else {
                                ShaderError::ParseError(p.span(), "Expected directive".to_string())
                                    .emit();
                                continue;
                            };

                            match directive.to_string().as_str() {
                                "vertex" => {
                                    if let Some(vertex_fn) = l.get(2) {
                                        vertex_fn_name = Some(vertex_fn.to_string());
                                    } else {
                                        ShaderError::ParseError(
                                            directive.span(),
                                            "Expected vertex function name".to_string(),
                                        )
                                        .emit();
                                    }
                                }
                                "fragment" => {
                                    if let Some(fragment_fn) = l.get(2) {
                                        frag_fn_name = Some(fragment_fn.to_string());
                                    } else {
                                        ShaderError::ParseError(
                                            directive.span(),
                                            "Expected fragment function name".to_string(),
                                        )
                                        .emit();
                                    }
                                }
                                "geometry" => {
                                    if let Some(geometry_fn) = l.get(2) {
                                        geometry_fn_name = Some(geometry_fn.to_string());
                                    } else {
                                        ShaderError::ParseError(
                                            directive.span(),
                                            "Expected geometry function name".to_string(),
                                        )
                                        .emit();
                                    }
                                }
                                _ => {}
                            }

                            // TODO: Handle other preprocessor directives
                            continue;
                        }
                    }
                    TokenTree::Literal(l) => {
                        ShaderError::ParseError(l.span(), "Unexpected Literal".to_string()).emit();
                        continue;
                    }
                    TokenTree::Ident(i) => {
                        if i.to_string() == "uniform" {
                            if let Some(var) = parse_var(&l[1..], &[], info) {
                                info.uniforms.push(var);
                            }
                        }
                    }
                    _ => {}
                }
            }
            Segment::Group(l, g) => {
                if l.is_empty() {
                    ShaderError::ParseError(g.span, "Unexpected Block".to_string()).emit();
                    continue;
                }
                let first = l
                    .first()
                    .expect("Block has no first element but is not empty");
                if first.to_string() == "struct" {
                    if let Some(name) = l.get(1) {
                        let shader_struct = parse_struct(name.to_string(), g, info);
                        info.structs.push(shader_struct);
                    } else {
                        ShaderError::ParseError(first.span(), "Expected struct name".to_string())
                            .emit();
                    }
                    continue;
                }

                let bracket = l
                    .get(2)
                    .filter(|b| match b {
                        TokenTree::Punct(p) => p.as_char() == '#',
                        _ => false,
                    })
                    .is_some();
                if bracket {
                    if let Some(function) = parse_function(l, g, info) {
                        if vertex_fn_name
                            .as_ref()
                            .is_some_and(|name| function.name == *name)
                        {
                            info.vertex_fn = Some(function);
                        } else if frag_fn_name
                            .as_ref()
                            .is_some_and(|name| function.name == *name)
                        {
                            info.frag_fn = Some(function);
                        } else if geometry_fn_name
                            .as_ref()
                            .is_some_and(|name| function.name == *name)
                        {
                            info.geometry_fn = Some(function);
                        } else {
                            info.functions.push(function);
                        }
                    }
                    continue;
                }
            }
        }
    }
}

fn parse_var(l: &[TokenTree], local_vars: &[ShaderVar], info: &ShaderInfo) -> Option<ShaderVar> {
    if l.is_empty() {
        return None;
    }

    let type_name = l
        .first()
        .expect("Line has no first element but is not empty");

    if let Some(name) = l.get(1) {
        let parsed_type = info.get_type(type_name, local_vars);
        Some(ShaderVar {
            name: name.to_string(),
            name_span: name.span(),
            t: parsed_type,
            type_span: Some(type_name.span()),
        })
    } else {
        ShaderError::ParseError(type_name.span(), "Expected field name".to_string()).emit();
        None
    }
}

fn parse_struct(name: String, block: &Block, info: &ShaderInfo) -> ShaderStruct {
    let name = name.to_string();
    let mut fields = vec![];

    for segment in &block.segments {
        match segment {
            Segment::Line(l) => {
                if let Some(var) = parse_var(l, &[], info) {
                    fields.push(var);
                }
            }
            Segment::Group(_, _) => {
                ShaderError::ParseError(block.span, "Unexpected block in struct".to_string())
                    .emit();
            }
        }
    }

    ShaderStruct { name, fields }
}

fn parse_function(line: &[TokenTree], block: &Block, info: &ShaderInfo) -> Option<ShaderFunction> {
    if line.is_empty() {
        return None;
    }

    let return_type = line.first().expect("Line has no first but is not empty");

    let name = if let Some(name) = line.get(1) {
        name
    } else {
        ShaderError::ParseError(return_type.span(), "Expected function name".to_string()).emit();
        return None;
    };

    let mut params = vec![];

    for param in line[3..].split(|i| match i {
        TokenTree::Punct(p) => p.as_char() == ',',
        _ => false,
    }) {
        if param.is_empty() {
            continue;
        }

        let type_name = &param[0];

        let name = if let Some(name) = param.get(1) {
            name
        } else {
            ShaderError::ParseError(type_name.span(), "Expected parameter name".to_string()).emit();
            continue;
        };

        let var_type = info.get_type(type_name, &[]);

        let var = ShaderVar::new(
            var_type,
            Some(type_name.span()),
            name.to_string(),
            name.span(),
        );

        params.push(var);
    }

    let content = block_to_source(block);
    parse_block(block, params.clone(), info);

    let return_type = info.get_type(return_type, &[]);

    Some(ShaderFunction {
        name: name.to_string(),
        params,
        return_type,
        content,
    })
}

fn parse_block(block: &Block, mut local_vars: Vec<ShaderVar>, info: &ShaderInfo) {
    for segment in &block.segments {
        match segment {
            Segment::Line(l) => {
                type_checking::type_check(l, &mut local_vars, info);
            }
            Segment::Group(_l, b) => {
                parse_block(b, local_vars.clone(), info);
            }
        }
    }
}

/// Build segments from a token stream
/// Seperates out while lines, and groups lines into any blocks they may be in
fn build_segments(input: Group) -> Vec<Segment> {
    let input = input.stream();

    let mut segments = vec![];

    let mut current_segment = vec![];

    let mut since_last_preprocessor = None;

    for token in input {
        if let Some(since_last_preprocessor) = since_last_preprocessor.as_mut() {
            *since_last_preprocessor += 1;
        }
        match &token {
            TokenTree::Punct(p) => {
                if p.as_char() == ';' {
                    segments.push(Segment::Line(current_segment.clone()));
                    current_segment.clear();
                    continue;
                } else {
                    current_segment.push(TokenTree::Punct(p.clone()));
                }

                if p.as_char() == '#' {
                    since_last_preprocessor = Some(0);
                }
            }
            TokenTree::Group(g) => {
                if g.delimiter() == Delimiter::Parenthesis {
                    let tokens: Vec<TokenTree> = g.stream().into_iter().collect();
                    current_segment.push(TokenTree::Punct(Punct::new('#', Spacing::Alone)));
                    current_segment.extend(tokens);
                } else {
                    let span = g.span();
                    let segs = build_segments(g.clone());
                    let block = Block {
                        span,
                        segments: segs,
                    };
                    segments.push(Segment::Group(current_segment.clone(), block));
                    current_segment.clear();
                }
            }
            _ => {
                current_segment.push(token.clone());
            }
        }
        if let Some(since) = since_last_preprocessor {
            if since >= 2 {
                segments.push(Segment::Line(current_segment.clone()));
                current_segment.clear();
                since_last_preprocessor = None;
            }
        }
    }

    segments
}

fn block_to_source(block: &Block) -> String {
    let source = block.span.source_text().unwrap_or_default();

    let source = format!(
        "    {}",
        source
            .trim()
            .trim_start_matches('{')
            .trim_end_matches('}')
            .trim()
    );

    source
}
