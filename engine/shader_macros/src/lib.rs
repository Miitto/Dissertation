#![feature(proc_macro_diagnostic, proc_macro_span)]
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use shader_info::ShaderInfo;
use shader_var::{ShaderObjects, ShaderType};

mod build_glsl;
mod errors;
mod parse_glsl;
mod parse_meta;
mod shader_info;
mod shader_var;
mod type_checking;

#[derive(Debug)]
struct ProgramMeta {
    name: String,
    version: i32,
}

impl ProgramMeta {
    pub fn ident(&self) -> proc_macro2::Ident {
        format_ident!("{}Program", self.name)
    }

    pub fn vertex_ident(&self) -> proc_macro2::Ident {
        format_ident!("{}Vertex", self.name)
    }

    pub fn uniforms_ident(&self) -> proc_macro2::Ident {
        format_ident!("{}Uniforms", self.name)
    }
}

#[proc_macro]
pub fn program(input: TokenStream) -> TokenStream {
    let mut iter = input.into_iter();

    let (meta, content) = parse_meta::parse_program_meta(&mut iter);

    let info = parse_glsl::parse_glsl(content);

    let vertex_shader = build_glsl::build_vertex_shader(&info, &meta);
    let fragment_shader = build_glsl::build_fragment_shader(&info, &meta);
    let geometry_shader = build_glsl::build_geometry_shader(&info, &meta);

    let vertex_struct = make_vertex_struct(meta.vertex_ident(), &info);
    let uniform_struct = make_uniform_struct(meta.uniforms_ident(), &info);

    let program_impl = make_program(
        meta.ident(),
        &vertex_shader,
        &fragment_shader,
        geometry_shader.as_ref(),
    );

    let expanded = quote! {
        #vertex_struct

        #uniform_struct

        #program_impl
    };

    expanded.into()
}

fn make_vertex_struct(ident: proc_macro2::Ident, info: &ShaderInfo) -> proc_macro2::TokenStream {
    let (fields, field_names) = if let Some(vertex_fn) = &info.vertex_fn {
        let in_param = vertex_fn
            .params
            .first()
            .expect("Vertex function must have one parameter");

        let vertex_input = match &in_param.t {
            ShaderType::Object(ShaderObjects::Custom(s)) => s,
            _ => panic!("Vertex function must take a struct as input"),
        };

        let fields = vertex_input
            .fields
            .iter()
            .map(|f| {
                let name = format_ident!("{}", f.name);
                let ty = &f.t;
                quote! {
                    #name: #ty
                }
            })
            .collect();
        let names = vertex_input
            .fields
            .iter()
            .map(|f| format_ident!("{}", f.name))
            .collect();

        (fields, names)
    } else {
        (vec![], vec![])
    };

    quote! {
        #[derive(Debug, Clone, Copy)]
        pub struct #ident {
            #(pub #fields),*
        }

        ::glium::implement_vertex!(#ident, #(#field_names),*);
    }
}

fn make_uniform_struct(ident: proc_macro2::Ident, info: &ShaderInfo) -> proc_macro2::TokenStream {
    let fields: Vec<proc_macro2::TokenStream> = info
        .uniforms
        .iter()
        .map(|uniform| {
            let name = format_ident!("{}", uniform.name);
            let ty = &uniform.t;
            quote! {
                #name: #ty
            }
        })
        .collect();

    quote! {
        pub struct #ident {
            #(pub #fields),*
        }
    }
}

fn make_program(
    ident: proc_macro2::Ident,
    vertex_source: &str,
    fragment_source: &str,
    geom_source: Option<&String>,
) -> proc_macro2::TokenStream {
    let geom_source = match geom_source {
        Some(s) => quote! { Some(#s) },
        None => quote! { None },
    };

    quote! {
        pub struct #ident;

        impl ::shaders::ProgramInternal for #ident {
            fn vertex() -> &'static str {
                #vertex_source
            }

            fn fragment() -> &'static str {
                #fragment_source
            }

            fn geometry() -> Option<&'static str> {
                #geom_source
            }
        }
    }
}
