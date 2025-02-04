use proc_macro::{Group, TokenTree};

use crate::{
    ProgramInput, ProgramMeta, ShaderError, ShaderInput, ShaderMeta,
    shader_var::{ShaderVar, ShaderVarType},
};

#[derive(Clone, Debug, Default)]
struct ShaderStruct {
    name: String,
    fields: Vec<ShaderVar>,
}

#[derive(Clone, Debug)]
struct ShaderFunction {
    pub return_type: ShaderVarType,
    pub name: String,
    pub params: Vec<ShaderVar>,
    pub content: String,
}

enum Segment {
    Line(Vec<TokenTree>),
    Group(Vec<TokenTree>, Vec<Segment>),
}

#[derive(Clone, Debug, Default)]
struct ShaderInfo {
    pub structs: Vec<ShaderStruct>,
    pub functions: Vec<ShaderFunction>,
    pub vertex_fn: Option<ShaderFunction>,
    pub frame_fn: Option<ShaderFunction>,
    pub geometry_fn: Option<ShaderFunction>,
}

pub fn parse_glsl(input: Group, meta: ProgramMeta) -> ProgramInput {
    let segments = build_segments(input);

    let mut info = ShaderInfo::default();
    let mut errors = vec![];
    let mut local_vars = vec![];

    parse_segments(0, segments, &mut info, &mut local_vars, &mut errors);
    todo!()
}

fn parse_segments(
    depth: u32,
    segments: Vec<Segment>,
    info: &mut ShaderInfo,
    local_vars: &mut Vec<ShaderVar>,
    errors: &mut Vec<ShaderError>,
) {
    let mut iterator = segments.into_iter().peekable();

    let next = iterator.next();
    while let Some(next) = next.as_ref() {
        match next {
            Segment::Line(l) => {}
            Segment::Group(_, _) => {}
        }
    }
}

fn parse_line(
    depth: u32,
    line: Vec<TokenTree>,
    info: &mut ShaderInfo,
    local_vars: &mut Vec<ShaderVar>,
    errors: &mut Vec<ShaderError>,
) {
    if line.is_empty() {
        return;
    }

    let first = line.first().unwrap();

    let first_string = first.to_string();

    fn check_type_mismatch(
        found: &ShaderVar,
        var: &TokenTree,
        equals: Option<&TokenTree>,
        assigner: Option<&TokenTree>,
        local_vars: &[ShaderVar],
        errors: &mut Vec<ShaderError>,
    ) {
        // Check for assignment
        if let Some(equals) = equals {
            if let TokenTree::Punct(p) = equals {
                if p.as_char() != '=' {
                    return;
                }
            }
            if let Some(third) = assigner {
                let as_string = third.to_string();

                // Check for existing variable being assinged to it
                let assign_found = local_vars.iter().find(|v| v.name == as_string);

                if let Some(assign_found) = assign_found {
                    if assign_found.r#type != found.r#type {
                        errors.push(ShaderError::TypeMismatch(
                            var.span(),
                            found.clone(),
                            assign_found.clone(),
                        ));
                    }
                    return;
                }

                // Check for creating another type, such as `vec2(1.0, 1.0)`
                let as_type: ShaderVarType = as_string.as_str().into();

                if let ShaderVarType::Other(_) = as_type {
                    return;
                }

                let this_var = ShaderVar::new(
                    as_type.clone(),
                    Some(third.span()),
                    var.to_string(),
                    var.span(),
                );

                if as_type != found.r#type {
                    errors.push(ShaderError::TypeMismatch(
                        var.span(),
                        found.clone(),
                        this_var,
                    ));
                }
            } else {
                errors.push(ShaderError::ParseError(
                    equals.span(),
                    "Expected value after '='".to_string(),
                ));
            }
        }
    }

    // Create a new variable if a line starts with a type
    if ShaderVarType::is_type(first_string.as_str()) {
        let var_type = first;
        let shader_type = var_type.to_string().into();
        let type_span = var_type.span();
        let var_name = line.get(1).expect("Failed to get var name");
        let name_string = var_name.to_string();
        let name_span = var_name.span();

        let shader_var = ShaderVar::new(shader_type, Some(type_span), name_string, name_span);

        local_vars.push(shader_var.clone());

        let equals = line.get(2);
        let assigner = line.get(3);

        check_type_mismatch(&shader_var, var_name, equals, assigner, local_vars, errors);

        return;
    }

    // Check for type mismatches
    let found = local_vars.iter().find(|v| v.name == first_string);
    if let Some(found) = found {
        check_type_mismatch(found, first, line.get(1), line.get(2), local_vars, errors);
    }
}

/// Build segments from a token stream
/// Seperates out while lines, and groups lines into any blocks they may be in
fn build_segments(input: Group) -> Vec<Segment> {
    let input = input.stream();

    let mut segments = vec![];

    let mut current_segment = vec![];

    for token in input {
        match token {
            TokenTree::Punct(p) => {
                if p.as_char() == ';' {
                    segments.push(Segment::Line(current_segment.clone()));
                    current_segment.clear();
                    continue;
                } else {
                    current_segment.push(TokenTree::Punct(p));
                }
            }
            TokenTree::Group(g) => {
                let segs = build_segments(g);
                segments.push(Segment::Group(current_segment.clone(), segs));
                current_segment.clear();
            }
            _ => {
                current_segment.push(token);
            }
        }
    }

    segments
}
