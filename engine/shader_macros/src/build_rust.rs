use crate::{
    ProgramInput,
    shader_var::{ShaderType, ShaderVar},
};
use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn vertex_struct(input: &ProgramInput, use_crate: bool) -> proc_macro2::TokenStream {
    let ProgramInput { content: info, .. } = input;

    let (fields, bind_count, binds) = if let Some(vertex_fn) = &info.vertex_fn {
        let in_param = vertex_fn
            .params
            .first()
            .expect("Vertex function must have one parameter");

        let vertex_input = match &in_param.t {
            ShaderType::Struct(s) => s,
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

        let (bind_count, binds) =
            vertex_binds(format_ident!("Vertex"), &vertex_input.fields, 0, use_crate);

        (fields, bind_count, binds)
    } else {
        (vec![], 0, quote! {})
    };

    let instance_struct =
        if let Some(param) = &info.vertex_fn.as_ref().and_then(|f| f.params.get(1)) {
            let vertex_input = match &param.t {
                ShaderType::Struct(s) => s,
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

            let (_, vertex_binds) = vertex_binds(
                format_ident!("Instance"),
                &vertex_input.fields,
                bind_count,
                use_crate,
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

pub fn uniform_struct(input: &ProgramInput, use_crate: bool) -> proc_macro2::TokenStream {
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

    let crate_path = if use_crate {
        quote! {crate}
    } else {
        quote! {renderer}
    };

    let binds = info.uniforms.iter().map(|uniform| {
        let name = format_ident!("{}", uniform.var.name.to_string());
        quote! {
            let loc = #crate_path::get_uniform_location(program, stringify!(#name));
            self.#name.set_uniform(loc);
        }
    });

    quote! {
        pub struct Uniforms {
            #(pub #fields),*
        }

        impl #crate_path::Uniforms for Uniforms {
            fn bind(&self, program: &#crate_path::Program) {
                    use #crate_path::UniformValue;

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
    use_crate: bool,
) -> proc_macro2::TokenStream {
    let geom_source = match geom_source {
        Some(s) => quote! { Some(#s) },
        None => quote! { None },
    };

    let crate_path = if use_crate {
        quote! {crate}
    } else {
        quote! {renderer}
    };

    quote! {
        pub struct Program;

        impl #crate_path::ProgramInternal for Program {
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

fn vertex_binds(
    ident: Ident,
    fields: &[ShaderVar],
    layout_start: usize,
    use_crate: bool,
) -> (usize, TokenStream) {
    let crate_path = if use_crate {
        quote! {crate}
    } else {
        quote! {renderer}
    };

    let mut loc = layout_start;
    let mut binds: Vec<TokenStream> = vec![];

    for field in fields {
        let ty = &field.t;
        let field_name = format_ident!("{}", &field.name.to_string());

        let primative = match ty {
            ShaderType::Primative(p) => p,
            _ => panic!("Can't yet handle structs in vertex structs"),
        };

        let layouts = primative.slots_taken();
        let size = primative.get_size_bytes();

        let layout_offset = size / layouts;

        let is_int = primative.is_integer();
        let elements = primative.get_num_components() / layouts as u32;

        for layout in 0..layouts {
            let tokens = quote! {
                #crate_path::vertex::format::VertexAtrib {
                    location: #loc as u32,
                    is_int: #is_int,
                    elements: #elements as i32,
                    ty: {const fn attr_type_of_val<T: #crate_path::vertex::Attribute>(_: Option<&T>)
                                -> #crate_path::vertex::format::AttributeType
                            {
                                <T as #crate_path::vertex::Attribute>::TYPE
                            }
                            attr_type_of_val(None::<&#ty>)
                    }.get_gl_primative(),
                    offset: (#crate_path::offset_of!(#ident, #field_name) + (#layout * #layout_offset)) as u32,
               }
            };

            binds.push(tokens);
            loc += 1;
        }
    }

    (
        loc,
        quote! {
            impl #ident {
                const BINDINGS: #crate_path::vertex::format::VertexFormat = &[
                    #(#binds),*
                ];
            }

            impl #crate_path::vertex::Vertex for #ident {
                fn bindings() -> #crate_path::vertex::format::VertexFormat {
                    Self::BINDINGS
                }
            }
        },
    )
}
