use crate::{
    ProgramInput,
    shader_var::{ShaderObjects, ShaderType},
};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

pub fn vertex_struct(input: &ProgramInput) -> proc_macro2::TokenStream {
    let ProgramInput { content: info, .. } = input;

    let (fields, field_names) = if let Some(vertex_fn) = &info.vertex_fn {
        let in_param = vertex_fn
            .params
            .first()
            .expect("Vertex function must have one parameter");

        let vertex_input = match &in_param.t {
            ShaderType::Object(ShaderObjects::Custom(s)) => s,
            _ => panic!("Vertex function must take structs as input"),
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

    let instance_struct =
        if let Some(param) = &info.vertex_fn.as_ref().and_then(|f| f.params.get(1)) {
            let vertex_input = match &param.t {
                ShaderType::Object(ShaderObjects::Custom(s)) => s,
                _ => panic!("Vertex function must take structs as input"),
            };

            let fields: Vec<TokenStream> = vertex_input
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

            let names: Vec<Ident> = vertex_input
                .fields
                .iter()
                .map(|f| format_ident!("{}", f.name))
                .collect();

            quote! {
                #[derive(Debug, Clone, Copy)]
                pub struct Instance {
                    #(pub #fields),*
                }

                ::glium::implement_vertex!(Instance, #(#names),*);
            }
        } else {
            quote! {}
        };

    quote! {
        #[derive(Debug, Clone, Copy)]
        pub struct Vertex {
            #(pub #fields),*
        }

        ::glium::implement_vertex!(Vertex, #(#field_names),*);

        #instance_struct
    }
}

pub fn uniform_struct(input: &ProgramInput) -> proc_macro2::TokenStream {
    let ProgramInput { content: info, .. } = input;

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
        pub struct Uniforms {
            #(pub #fields),*
        }
    }
}

pub fn program(
    vertex_source: &str,
    fragment_source: &str,
    geom_source: Option<&String>,
) -> proc_macro2::TokenStream {
    let geom_source = match geom_source {
        Some(s) => quote! { Some(#s) },
        None => quote! { None },
    };

    quote! {
        pub struct Program;

        impl ::shaders::ProgramInternal for Program {
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
