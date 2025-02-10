#![feature(proc_macro_diagnostic, proc_macro_span)]
use std::{fs::OpenOptions, io::Write};

use parse::meta::ProgramMeta;
use proc_macro::TokenStream;
use quote::quote;
use shader_info::ShaderInfo;
use syn::{
    braced,
    parse::{ParseBuffer, ParseStream},
    parse_macro_input,
};

mod build_glsl;
mod build_rust;
mod errors;
mod parse;
mod shader_info;
mod shader_var;

struct ProgramInput {
    meta: ProgramMeta,
    content: ShaderInfo,
}

impl syn::parse::Parse for ProgramInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let meta = input.parse()?;
        let content: ParseBuffer;
        _ = braced!(content in input);

        let info = content.parse()?;

        Ok(Self {
            meta,
            content: info,
        })
    }
}

#[proc_macro]
pub fn program(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ProgramInput);

    let vertex_shader = build_glsl::vertex_shader(&input);
    let fragment_shader = build_glsl::fragment_shader(&input);
    let geometry_shader = build_glsl::geometry_shader(&input);

    let vertex_struct = build_rust::vertex_struct(&input);
    let uniform_struct = build_rust::uniform_struct(&input);

    let program_impl =
        build_rust::program(&vertex_shader, &fragment_shader, geometry_shader.as_ref());

    let name = input.meta.name;

    let expanded = quote! {
        pub mod #name {
        #vertex_struct

        #uniform_struct

        #program_impl
        }
    };

    expanded.into()
}
