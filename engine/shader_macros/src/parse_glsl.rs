use std::fmt::Display;

use proc_macro::{Delimiter, Group, Punct, Spacing, Span, TokenTree};

use crate::{
    errors::ShaderError,
    link_info::{LinkedShaderInfo, link_info},
    shader_var::{ShaderVar, ShaderVarType},
};

#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
pub(crate) struct ShaderStruct {
    pub name: String,
    pub fields: Vec<ShaderVar>,
}

impl Display for ShaderStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "struct {} {{\n{}\n}};",
            self.name,
            self.fields
                .iter()
                .map(|f| format!("    {};", f))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(crate) struct ShaderFunction {
    pub return_type: ShaderVarType,
    pub name: String,
    pub params: Vec<ShaderVar>,
    pub content: String,
}

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

#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
pub(crate) struct ShaderInfo {
    pub structs: Vec<ShaderStruct>,
    pub functions: Vec<ShaderFunction>,
    pub global_vars: Vec<ShaderVar>,
    pub uniforms: Vec<ShaderVar>,
    pub vertex_fn: Option<String>,
    pub frag_fn: Option<String>,
    pub geometry_fn: Option<String>,
}

pub fn parse_glsl(input: Group) -> (LinkedShaderInfo, Vec<ShaderError>) {
    let segments = build_segments(input);

    let mut info = ShaderInfo::default();
    let mut errors = vec![];

    parse_segments(&segments, &mut info, &mut errors);

    (link_info(info), errors)
}

fn parse_segments(segments: &[Segment], info: &mut ShaderInfo, errors: &mut Vec<ShaderError>) {
    let mut iterator = segments.iter().peekable();

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
                                errors.push(ShaderError::ParseError(
                                    p.span(),
                                    "Expected directive".to_string(),
                                ));
                                continue;
                            };

                            match directive.to_string().as_str() {
                                "vertex" => {
                                    if let Some(vertex_fn) = l.get(2) {
                                        info.vertex_fn = Some(vertex_fn.to_string());
                                    } else {
                                        errors.push(ShaderError::ParseError(
                                            directive.span(),
                                            "Expected vertex function name".to_string(),
                                        ));
                                    }
                                }
                                "fragment" => {
                                    if let Some(fragment_fn) = l.get(2) {
                                        info.frag_fn = Some(fragment_fn.to_string());
                                    } else {
                                        errors.push(ShaderError::ParseError(
                                            directive.span(),
                                            "Expected fragment function name".to_string(),
                                        ));
                                    }
                                }
                                "geometry" => {
                                    if let Some(geometry_fn) = l.get(2) {
                                        info.geometry_fn = Some(geometry_fn.to_string());
                                    } else {
                                        errors.push(ShaderError::ParseError(
                                            directive.span(),
                                            "Expected geometry function name".to_string(),
                                        ));
                                    }
                                }
                                _ => {}
                            }

                            // TODO: Handle other preprocessor directives
                            continue;
                        }
                    }
                    TokenTree::Literal(l) => {
                        errors.push(ShaderError::ParseError(
                            l.span(),
                            "Unexpected Literal".to_string(),
                        ));
                        continue;
                    }
                    TokenTree::Ident(i) => {
                        if i.to_string() == "uniform" {
                            if let Some(var) = parse_var(&l[1..], errors) {
                                info.global_vars.push(var.clone());
                                info.uniforms.push(var);
                            }
                        }

                        if let Some(var) = parse_var(l, errors) {
                            info.global_vars.push(var);
                        }
                    }
                    _ => {}
                }
            }
            Segment::Group(l, g) => {
                if l.is_empty() {
                    errors.push(ShaderError::ParseError(
                        g.span,
                        "Unexpected Block".to_string(),
                    ));
                    continue;
                }
                let first = l
                    .first()
                    .expect("Block has no first element but is not empty");
                if first.to_string() == "struct" {
                    if let Some(name) = l.get(1) {
                        let shader_struct = parse_struct(name.to_string(), g, errors);
                        info.structs.push(shader_struct);
                    } else {
                        errors.push(ShaderError::ParseError(
                            first.span(),
                            "Expected struct name".to_string(),
                        ));
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
                    if let Some(function) = parse_function(l, g, info, errors) {
                        info.functions.push(function);
                    }
                    continue;
                }
            }
        }
    }
}

fn parse_var(l: &[TokenTree], errors: &mut Vec<ShaderError>) -> Option<ShaderVar> {
    if l.is_empty() {
        return None;
    }

    let type_name = l
        .first()
        .expect("Line has no first element but is not empty");

    if let Some(name) = l.get(1) {
        let parsed_type = ShaderVarType::from(type_name.to_string());
        Some(ShaderVar {
            name: name.to_string(),
            name_span: name.span(),
            r#type: parsed_type,
            type_span: Some(type_name.span()),
        })
    } else {
        errors.push(ShaderError::ParseError(
            type_name.span(),
            "Expected field name".to_string(),
        ));
        None
    }
}

fn parse_struct(name: String, block: &Block, errors: &mut Vec<ShaderError>) -> ShaderStruct {
    let name = name.to_string();
    let mut fields = vec![];

    for segment in &block.segments {
        match segment {
            Segment::Line(l) => {
                if let Some(var) = parse_var(l, errors) {
                    fields.push(var);
                }
            }
            Segment::Group(_, _) => {
                errors.push(ShaderError::ParseError(
                    block.span,
                    "Unexpected block in struct".to_string(),
                ));
            }
        }
    }

    ShaderStruct { name, fields }
}

fn parse_function(
    line: &[TokenTree],
    block: &Block,
    info: &ShaderInfo,
    errors: &mut Vec<ShaderError>,
) -> Option<ShaderFunction> {
    if line.is_empty() {
        return None;
    }

    let return_type = line.first().expect("Line has no first but is not empty");

    let name = if let Some(name) = line.get(1) {
        name
    } else {
        errors.push(ShaderError::ParseError(
            return_type.span(),
            "Expected function name".to_string(),
        ));
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

        let type_name = if let Some(type_name) = param.first() {
            type_name
        } else {
            unreachable!();
        };

        let name = if let Some(name) = param.get(1) {
            name
        } else {
            errors.push(ShaderError::ParseError(
                type_name.span(),
                "Expected parameter name".to_string(),
            ));
            continue;
        };

        let var = ShaderVar::new(
            type_name.to_string().into(),
            Some(type_name.span()),
            name.to_string(),
            name.span(),
        );

        params.push(var);
    }

    let content = block_to_source(block);
    parse_block(block, vec![], info, errors);

    Some(ShaderFunction {
        name: name.to_string(),
        params,
        return_type: return_type.to_string().into(),
        content,
    })
}

fn parse_block(
    block: &Block,
    mut local_vars: Vec<ShaderVar>,
    info: &ShaderInfo,
    errors: &mut Vec<ShaderError>,
) {
    fn get_type(
        name: String,
        span: Span,
        local_vars: &[ShaderVar],
        info: &ShaderInfo,
        errors: &mut Vec<ShaderError>,
    ) -> Option<ShaderVarType> {
        let into = ShaderVarType::from(&name);
        // Check builtin
        if !matches!(into, ShaderVarType::Other(_)) {
            return Some(into);
        }

        // Check structs
        if let Some(_t) = info.structs.iter().find(|s| s.name == name) {
            return Some(ShaderVarType::Other(name));
        }

        if let Some(var) = local_vars.iter().find(|v| v.name == name) {
            return Some(var.r#type.clone());
        }

        for var in info.global_vars.iter() {
            if var.name == name {
                return Some(var.r#type.clone());
            }
        }

        errors.push(ShaderError::UnknownType(span, name));

        None
    }

    fn get_var(name: &String, local_vars: &[ShaderVar], info: &ShaderInfo) -> Option<ShaderVar> {
        if let Some(var) = local_vars.iter().find(|v| &v.name == name) {
            return Some(var.clone());
        }

        for var in info.global_vars.iter() {
            if &var.name == name {
                return Some(var.clone());
            }
        }

        None
    }

    fn is_assignment(line: &[TokenTree]) -> bool {
        if let Some(TokenTree::Punct(p)) = line.get(1) {
            p.as_char() == '='
        } else {
            false
        }
    }

    fn is_initialization(line: &[TokenTree]) -> bool {
        if let Some(TokenTree::Punct(p)) = line.get(2) {
            p.as_char() == '='
        } else {
            false
        }
    }

    fn check_assignment(
        line: &[TokenTree],
        local_vars: &[ShaderVar],
        info: &ShaderInfo,
        errors: &mut Vec<ShaderError>,
    ) {
        let name = if let Some(TokenTree::Ident(t)) = line.first() {
            t.to_string()
        } else {
            return;
        };

        if let Some(var) = get_var(&name, local_vars, info) {
            let assigned_type = if let Some(TokenTree::Ident(t)) = line.get(2) {
                get_type(t.to_string(), line[2].span(), local_vars, info, errors)
            } else {
                None
            };

            if let Some(assigned_type) = assigned_type {
                if var.r#type != assigned_type {
                    errors.push(ShaderError::TypeMismatch(
                        line[2].span(),
                        var.clone(),
                        ShaderVar {
                            name: line[2].to_string(),
                            r#type: assigned_type,
                            name_span: line[2].span(),
                            type_span: None,
                        },
                    ));
                }
            }
        }
    }

    fn check_initialization(
        line: &[TokenTree],
        local_vars: &mut Vec<ShaderVar>,
        info: &ShaderInfo,
        errors: &mut Vec<ShaderError>,
    ) {
        if let Some(var) = parse_var(line, errors) {
            local_vars.push(var);

            check_assignment(&line[1..], local_vars, info, errors);
        }
    }

    for segment in &block.segments {
        match segment {
            Segment::Line(l) => {
                if is_assignment(l) {
                    check_assignment(l, &local_vars, info, errors);
                }

                if is_initialization(l) {
                    check_initialization(l, &mut local_vars, info, errors);
                }
            }
            Segment::Group(_l, b) => {
                parse_block(b, local_vars.clone(), info, errors);
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
