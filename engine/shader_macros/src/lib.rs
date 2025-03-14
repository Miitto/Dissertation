#![feature(proc_macro_diagnostic, proc_macro_span)]
use parse::{Delimited, delimited, ident, meta::ProgramMeta, punct};
use proc_macro::{Delimiter, Diagnostic, Level, TokenStream};
use quote::{format_ident, quote};
use shader_info::ShaderInfo;

mod build_glsl;
mod build_rust;
mod parse;
mod shader_info;
mod shader_var;
mod uniform;

struct ProgramInput {
    meta: ProgramMeta,
    content: ShaderInfo,
}

type Result<I, C, E = ()> = core::result::Result<(I, C), E>;

fn abandon(name: proc_macro2::Ident) -> TokenStream {
    quote! {
    pub mod #name { pub struct Instance; pub struct Vertex; pub struct Program;}}
    .into()
}

#[proc_macro]
pub fn program(input: TokenStream) -> TokenStream {
    let collected: Vec<proc_macro::TokenTree> = input.into_iter().collect();

    let (content, meta) = match parse::meta::parse_meta(&collected) {
        Ok(parsed) => parsed,
        Err(e) => {
            e.emit();

            panic!("Failed to parse program metadata");
        }
    };

    let name = format_ident!("{}", meta.name.to_string());

    let (rest, Delimited { content, .. }) =
        if let Ok(content) = delimited(Delimiter::Brace)(content) {
            content
        } else {
            Diagnostic::spanned(content[0].span(), Level::Error, "Expected block").emit();
            return abandon(name);
        };

    let (_, info) = match parse::glsl::parse_glsl(&content) {
        Ok(parsed) => parsed,
        Err(e) => {
            e.emit();
            return abandon(name);
        }
    };

    let input = ProgramInput {
        meta,
        content: info,
    };

    let use_crate = punct(',')(rest)
        .and_then(|(rest, _)| ident("true")(rest))
        .is_ok();

    let vertex_shader = build_glsl::vertex_shader(&input);
    let fragment_shader = build_glsl::fragment_shader(&input);

    let vertex_struct = build_rust::vertex_struct(&input, use_crate);
    let uniform_struct = build_rust::uniform_struct(&input, use_crate);

    let program_impl = build_rust::program(
        &vertex_shader,
        &fragment_shader,
        None,
        &input.content.uses,
        input.meta.version,
        use_crate,
    );

    let uses = build_rust::uses(&input.content);

    let name = format_ident!("{}", input.meta.name.to_string());

    let includes = input.content.includes;

    quote! {
        pub mod #name {
            #(const _: &str = include_str!(#includes);)*

            #vertex_struct

            #uniform_struct

            #program_impl

            #uses
        }
    }
    .into()
}

#[proc_macro]
pub fn snippet(input: TokenStream) -> TokenStream {
    let collected: Vec<proc_macro::TokenTree> = input.into_iter().collect();

    let (content, meta) = match parse::meta::parse_meta(&collected) {
        Ok(parsed) => parsed,
        Err(e) => {
            e.emit();

            panic!("Failed to parse program metadata");
        }
    };

    let name = format_ident!("{}", meta.name.to_string());

    let (rest, Delimited { content, .. }) =
        if let Ok(content) = delimited(Delimiter::Brace)(content) {
            content
        } else {
            Diagnostic::spanned(content[0].span(), Level::Error, "Expected block").emit();
            return abandon(name);
        };

    let (_, info) = match parse::glsl::parse_glsl(&content) {
        Ok(parsed) => parsed,
        Err(e) => {
            e.emit();
            return abandon(name);
        }
    };

    let use_crate = punct(',')(rest)
        .and_then(|(rest, _)| ident("true")(rest))
        .is_ok();

    let input = ProgramInput {
        meta,
        content: info,
    };

    let uniform_struct = build_rust::uniform_struct(&input, use_crate);

    let source = build_glsl::no_main(&input);

    quote! {
        pub mod #name {
            #uniform_struct

            pub const SOURCE: &'static str = #source;
        }
    }
    .into()
}
