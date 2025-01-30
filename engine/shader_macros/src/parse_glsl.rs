use proc_macro::{Group, TokenTree};

use crate::{
    ShaderError, ShaderInput, ShaderMeta,
    shader_var::{ShaderVar, ShaderVarType},
};

enum Segment {
    Line(Vec<TokenTree>),
    #[allow(dead_code)]
    Group(Vec<TokenTree>, Vec<Segment>),
}

pub fn parse_glsl(input: Group, meta: ShaderMeta) -> ShaderInput {
    let mut shader_in = vec![];
    let mut shader_out = vec![];
    let mut shader_uniforms = vec![];

    let (content, segments) = build_segments(input, true);

    let content = format!("#version {}\n{}", meta.version, content);

    let mut local_vars = vec![];
    let mut errors = vec![];

    for segment in segments {
        parse_segment(
            0,
            segment,
            &mut shader_in,
            &mut shader_out,
            &mut shader_uniforms,
            &mut local_vars,
            &mut errors,
        );
    }

    ShaderInput {
        content,
        shader_in,
        shader_out,
        shader_uniforms,
        errors,
    }
}

fn parse_segment(
    depth: u32,
    segment: Segment,
    shader_in: &mut Vec<ShaderVar>,
    shader_out: &mut Vec<ShaderVar>,
    shader_uniforms: &mut Vec<ShaderVar>,
    local_vars: &mut Vec<ShaderVar>,
    errors: &mut Vec<ShaderError>,
) {
    match segment {
        Segment::Line(l) => {
            parse_line(
                depth,
                l,
                shader_in,
                shader_out,
                shader_uniforms,
                local_vars,
                errors,
            );
        }
        Segment::Group(pre_block, group) => {
            parse_line(
                depth,
                pre_block,
                shader_in,
                shader_out,
                shader_uniforms,
                local_vars,
                errors,
            );

            for segment in group {
                parse_segment(
                    depth + 1,
                    segment,
                    shader_in,
                    shader_out,
                    shader_uniforms,
                    local_vars,
                    errors,
                );
            }
        }
    }
}

fn parse_line(
    depth: u32,
    line: Vec<TokenTree>,
    shader_in: &mut Vec<ShaderVar>,
    shader_out: &mut Vec<ShaderVar>,
    shader_uniforms: &mut Vec<ShaderVar>,
    local_vars: &mut Vec<ShaderVar>,
    errors: &mut Vec<ShaderError>,
) {
    if line.is_empty() {
        return;
    }

    let first = line.first().unwrap();

    let first_string = first.to_string();

    // Make in / out / uniforms
    if first_string == "in" || first_string == "out" || first_string == "uniform" {
        let var_type = line.get(1).expect("Failed to get var type");
        let shader_type = var_type.to_string().into();
        let type_span = var_type.span();
        let var_name = line.get(2).expect("Failed to get var name");
        let name_string = var_name.to_string();
        let name_span = var_name.span();

        let shader_var = ShaderVar::new(shader_type, Some(type_span), name_string, name_span);

        // If we are in a block, error
        if depth != 0 {
            errors.push(ShaderError::NestedInOutUniform(
                first.span(),
                shader_var.clone(),
            ));
        }

        // Also assign as a local var
        local_vars.push(shader_var.clone());

        match first_string.as_str() {
            "in" => shader_in.push(shader_var),
            "out" => shader_out.push(shader_var),
            "uniform" => shader_uniforms.push(shader_var),
            _ => unreachable!(),
        }
    }
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
fn build_segments(input: Group, get_source: bool) -> (String, Vec<Segment>) {
    let input = input.stream();

    let mut segments = vec![];

    let mut content = String::new();
    let mut current_segment = vec![];

    for token in input {
        if get_source {
            if let TokenTree::Punct(_) = &token {
            } else if content
                .chars()
                .last()
                .filter(|c| !c.is_whitespace())
                .is_some()
            {
                content.push(' ');
            }
            let source = token.span().source_text().unwrap_or(token.to_string());
            content.push_str(&source);
            if let TokenTree::Punct(p) = &token {
                if p.as_char() == ';' {
                    content.push('\n');
                }
            }
        }

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
                let (_, segs) = build_segments(g, false);
                segments.push(Segment::Group(current_segment.clone(), segs));
                current_segment.clear();
            }
            _ => {
                current_segment.push(token);
            }
        }
    }

    (content, segments)
}
