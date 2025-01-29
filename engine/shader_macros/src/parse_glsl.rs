use proc_macro::{Group, TokenTree};

use crate::{
    ShaderInput, ShaderMeta,
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

    for segment in segments {
        parse_segment(
            segment,
            &mut shader_in,
            &mut shader_out,
            &mut shader_uniforms,
        );
    }

    ShaderInput {
        meta,
        content,
        shader_in,
        shader_out,
        shader_uniforms,
    }
}

fn parse_segment(
    segment: Segment,
    shader_in: &mut Vec<ShaderVar>,
    shader_out: &mut Vec<ShaderVar>,
    shader_uniforms: &mut Vec<ShaderVar>,
) {
    match segment {
        Segment::Line(l) => {
            parse_line(l, shader_in, shader_out, shader_uniforms);
        }
        Segment::Group(_, _) => {}
    }
}

fn parse_line(
    line: Vec<TokenTree>,
    shader_in: &mut Vec<ShaderVar>,
    shader_out: &mut Vec<ShaderVar>,
    shader_uniforms: &mut Vec<ShaderVar>,
) {
    if line.is_empty() {
        return;
    }

    let first = line.first().unwrap();

    let first_string = first.to_string();

    if first_string == "in" || first_string == "out" || first_string == "uniform" {
        let var_type: ShaderVarType = line
            .get(1)
            .expect("Failed to get var type")
            .to_string()
            .into();
        let var_name = line.get(2).expect("Failed to get var name").to_string();

        let var = ShaderVar::new(var_type, var_name);

        match first_string.as_str() {
            "in" => shader_in.push(var),
            "out" => shader_out.push(var),
            "uniform" => shader_uniforms.push(var),
            _ => unreachable!(),
        }
    }
}

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
