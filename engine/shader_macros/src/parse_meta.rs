use proc_macro::{Group, TokenTree};

use crate::ProgramMeta;

pub fn parse_program_meta(
    input: &mut impl Iterator<Item = TokenTree>,
) -> (ProgramMeta, Group, Group, Option<Group>) {
    let program_name = input
        .next()
        .expect("Program needs to have a name")
        .to_string();
    input.next().expect("Missing comma");

    let mut shader_version: Option<i32> = None;

    let mut next = input.next();

    if let Some(TokenTree::Literal(l)) = next {
        shader_version = Some(
            l.to_string()
                .parse()
                .expect("Failed to parse shader version"),
        );
        input.next().expect("Missing comma");
        next = input.next();
    }

    let vertex_shader = next
        .and_then(|item| match item {
            TokenTree::Group(g) => Some(g),
            _ => None,
        })
        .expect("Vertex Shader is missing its content");

    input.next().expect("Missing comma");

    let fragment_shader = input
        .next()
        .and_then(|item| match item {
            TokenTree::Group(g) => Some(g),
            _ => None,
        })
        .expect("Fragment Shader is missing its content");

    _ = input.next();

    let geometry_shader = input.next().and_then(|item| match item {
        TokenTree::Group(g) => Some(g),
        _ => None,
    });

    (
        ProgramMeta {
            name: program_name,
            version: shader_version.unwrap_or(330),
        },
        vertex_shader,
        fragment_shader,
        geometry_shader,
    )
}
