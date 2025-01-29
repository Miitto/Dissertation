use proc_macro::{Group, TokenTree};
use shaders_common::ShaderType;

use crate::ShaderMeta;

pub fn parse_meta(input: &mut impl Iterator<Item = TokenTree>) -> (ShaderMeta, Group) {
    let shader_type = input.next().expect("Shader needs have a type");
    _ = input.next().expect("Missing comma");
    let shader_name = input
        .next()
        .expect("Shader needs to have a name")
        .to_string();
    _ = input.next().expect("Missing comma");
    let mut shader_version: Option<i32> = None;
    let mut shader_content = None;

    match input.next() {
        Some(item) => match item {
            TokenTree::Group(g) => {
                shader_content = Some(g);
            }
            TokenTree::Literal(l) => {
                shader_version = Some(
                    l.to_string()
                        .parse()
                        .expect("Failed to parse shader version"),
                );
            }
            _ => {
                panic!("Invalid shader content");
            }
        },
        None => {
            panic!("Shader is missing its content");
        }
    }

    if shader_content.is_none() {
        input.next();
        shader_content = input.next().and_then(|item| match item {
            TokenTree::Group(g) => Some(g),
            _ => None,
        });
    }

    let shader_type: ShaderType = shader_type
        .to_string()
        .try_into()
        .expect("Invalid Shader Type");

    (
        ShaderMeta {
            t: shader_type,
            name: shader_name,
            version: shader_version.unwrap_or(330),
        },
        shader_content.expect("Shader is missing its content"),
    )
}
