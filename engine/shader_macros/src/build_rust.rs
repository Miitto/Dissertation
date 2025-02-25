use crate::{
    ProgramInput,
    shader_var::{ShaderObjects, ShaderType, ShaderVar},
};
use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn vertex_struct(input: &ProgramInput) -> proc_macro2::TokenStream {
    let ProgramInput { content: info, .. } = input;

    let mut vertex_in_count = 0;

    let (fields, binds) = if let Some(vertex_fn) = &info.vertex_fn {
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
                let name = format_ident!("{}", f.name.to_string());
                let ty = &f.t;
                quote! {
                    #name: #ty
                }
            })
            .collect();

        vertex_in_count = vertex_input.fields.len();

        let binds = vertex_binds(format_ident!("Vertex"), &vertex_input.fields, 0);

        (fields, binds)
    } else {
        (vec![], quote! {})
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
                    let name = format_ident!("{}", f.name.to_string());
                    let ty = &f.t;
                    quote! {
                        #name: #ty
                    }
                })
                .collect();

            let vertex_binds = vertex_binds(
                format_ident!("Instance"),
                &vertex_input.fields,
                vertex_in_count,
            );

            quote! {
                #[derive(Debug, Clone, Copy)]
                pub struct Instance {
                    #(pub #fields),*
                }

                #vertex_binds
            }
        } else {
            quote! {}
        };

    quote! {
        #[derive(Debug, Clone, Copy)]
        pub struct Vertex {
            #(pub #fields),*
        }

        #binds

        #instance_struct
    }
}

pub fn uniform_struct(input: &ProgramInput) -> proc_macro2::TokenStream {
    let ProgramInput { content: info, .. } = input;

    let fields: Vec<proc_macro2::TokenStream> = info
        .uniforms
        .iter()
        .map(|uniform| {
            let name = format_ident!("{}", uniform.var.name.to_string());
            let ty = &uniform.var.t;
            if uniform.value.is_some() {
                quote! {
                #name: Option<#ty>
                }
            } else {
                quote! {
                    #name: #ty
                }
            }
        })
        .collect();

    let binds = info.uniforms.iter().map(|uniform| {
        let name = format_ident!("{}", uniform.var.name.to_string());
        quote! {
            let loc = renderer::get_uniform_location(program, stringify!(#name));
            self.#name.set_uniform(loc);
        }
    });

    quote! {
        pub struct Uniforms {
            #(pub #fields),*
        }

        impl renderer::Uniforms for Uniforms {
            fn bind(&self, program: &renderer::Program) {
                    use renderer::UniformValue;

                    program.bind();

                    #(#binds)*
            }
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

fn vertex_binds(ident: Ident, fields: &[ShaderVar], layout_start: usize) -> TokenStream {
    let fields: Vec<TokenStream> = fields
        .iter()
        .enumerate()
        .map(|(loc, field)| {
            let loc = loc + layout_start;
            let ty = &field.t;
            let field_name = format_ident!("{}", &field.name.to_string());
            quote! {
                renderer::vertex::format::VertexAtrib {
                    location: #loc,
                    ty: {const fn attr_type_of_val<T: renderer::vertex::Attribute>(_: Option<&T>)
                                -> renderer::vertex::format::AttributeType
                            {
                                <T as renderer::vertex::Attribute>::TYPE
                            }
                            attr_type_of_val(None::<&#ty>)
                    },
                    offset: renderer::offset_of!(#ident, #field_name)
               }
            }
        })
        .collect();

    quote! {
        impl #ident {
            const BINDINGS: renderer::vertex::format::VertexFormat = &[
                #(#fields),*
            ];
        }

        impl renderer::vertex::Vertex for #ident {
            fn bindings() -> renderer::vertex::format::VertexFormat {
                Self::BINDINGS
            }
        }
    }
}
