use crate::{
    ProgramInput, ShaderInfo,
    shader_var::{ShaderType, ShaderVar},
    uniform::Uniform,
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
        .filter_map(|u| {
            if let Uniform::Single(s) = &u {
                Some(s)
            } else {
                None
            }
        })
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

    let binds = info
        .uniforms
        .iter()
        .filter_map(|u| {
            if let Uniform::Single(s) = &u {
                Some(s)
            } else {
                None
            }
        })
        .map(|uniform| {
            let name = format_ident!("{}", uniform.var.name.to_string());
            quote! {
                let loc = #crate_path::get_uniform_location(program, stringify!(#name));
                self.#name.set_uniform(loc);
            }
        });

    let blocks = uniform_block_structs(input, use_crate);

    let buffers = buffer_structs(input, use_crate);

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

        pub mod uniforms {
            #blocks
        }

        pub mod buffers {
            #buffers
        }
    }
}

fn uniform_block_structs(input: &ProgramInput, use_crate: bool) -> proc_macro2::TokenStream {
    let blocks = input
        .content
        .uniforms
        .iter()
        .filter_map(|u| {
            if let Uniform::Block(b) = &u {
                Some(b)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if blocks.is_empty() {
        return quote! {};
    }

    let crate_path = if use_crate {
        quote! {crate}
    } else {
        quote! {renderer}
    };

    let structs = blocks
        .into_iter()
        .map(|u| {
            let name = format_ident!("{}", u.name.to_string());
            let fields = u.fields.iter().map(|f| {
                let ty = &f.t;
                let name = format_ident!("{}", f.name.to_string());

                quote! {#name: #ty}
            });

            let bind = u.bind;

            let field_names = u.fields.iter().map(|f| {
                format_ident!("{}", f.name.to_string())
            });

            let setters = field_names.map(|f| {
                quote! {
                    buffer.set_offset_data_no_alloc(offset, &[self.#f])?;
                    let align = attr_type_of_val(&self.#f).std140_align();
                    offset += align;
                }
            });

            let size: usize = u.fields.iter().map(|f| match &f.t {
                ShaderType::Primative(a) => a.std140_align(),
                ShaderType::Struct(_) => todo!("Get size for std140 struct"),
                ShaderType::Void => 0
            }).sum();

            quote! {
                #[derive(Debug)]
                pub struct #name {
                    #(pub #fields),*
                }

                impl #name {
                    const BIND: u32 = #bind;
                    const SIZE: usize = #size;
                }

                impl #crate_path::LayoutBlock for #name {
                    fn bind_point() -> u32 {
                        Self::BIND
                    }

                    fn size() -> usize {
                        Self::SIZE
                    }

                    fn set_buffer_data<B: #crate_path::buffers::RawBuffer>(&self, buffer: &mut B) -> Result<(), #crate_path::buffers::BufferError> {
                        let mut offset = 0;

                        const fn attr_type_of_val<T: #crate_path::vertex::Attribute>(_: &T)
                                -> #crate_path::vertex::format::AttributeType
                            {
                                <T as #crate_path::vertex::Attribute>::TYPE
                            }

                        #(#setters)*

                        Ok(())
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    quote! {
        #(#structs)*
    }
}

fn buffer_structs(input: &ProgramInput, use_crate: bool) -> proc_macro2::TokenStream {
    let blocks = &input.content.buffers;

    if blocks.is_empty() {
        return quote! {};
    }

    let crate_path = if use_crate {
        quote! {crate}
    } else {
        quote! {renderer}
    };

    let structs = blocks
        .iter()
        .map(|u| {
            let name = format_ident!("{}", u.name.to_string());
            let fields = u.fields.iter().map(|f| {
                let ty = &f.t;
                let name = format_ident!("{}", f.name.to_string());

                quote! {#name: #ty}
            });

            let bind = u.bind;

            let field_names = u.fields.iter().map(|f| {
                format_ident!("{}", f.name.to_string())
            });

            let setters = field_names.map(|f| {
                quote! {
                    buffer.set_offset_data_no_alloc(offset, &[self.#f])?;
                    let align = attr_type_of_val(&self.#f).std430_align();
                    offset += align;
                }
            });

            let size: usize = u.fields.iter().map(|f| match &f.t {
                ShaderType::Primative(a) => a.std430_align(),
                ShaderType::Struct(_) => todo!("Get size for std430 struct"),
                ShaderType::Void => 0
            }).sum();

            quote! {
                #[derive(Debug)]
                pub struct #name {
                    #(pub #fields),*
                }

                impl #name {
                    const BIND: u32 = #bind;
                    const SIZE: usize = #size;
                }

                impl #crate_path::LayoutBlock for #name {
                    fn bind_point() -> u32 {
                        Self::BIND
                    }

                    fn size() -> usize {
                        Self::SIZE
                    }

                    fn set_buffer_data<B: #crate_path::buffers::RawBuffer>(&self, buffer: &mut B) -> Result<(), #crate_path::buffers::BufferError> {
                        let mut offset = 0;

                        const fn attr_type_of_val<T: #crate_path::vertex::Attribute>(_: &T)
                                -> #crate_path::vertex::format::AttributeType
                            {
                                <T as #crate_path::vertex::Attribute>::TYPE
                            }

                        #(#setters)*

                        Ok(())
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    quote! {
        #(#structs)*
    }
}

pub fn program(
    vertex_source: &str,
    fragment_source: &str,
    geom_source: Option<&String>,
    uses: &[Vec<proc_macro::Ident>],
    version: u32,
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

    let uses = uses
        .iter()
        .map(|s| {
            let idents = s.iter().map(|i| format_ident!("{}", i.to_string()));
            quote! {#(#idents)::*::SOURCE}
        })
        .reduce(|acc, el| quote! {combine!(#acc, #el)})
        .unwrap_or(quote! {"\n"});

    let version = format!("#version {}\n", version);

    quote! {
        pub struct Program;

        impl #crate_path::ProgramInternal for Program {
            fn vertex() -> &'static str {
                macro_rules! combine {
                    ($A:expr, $B:expr) => {{
                        const A: &str = $A;
                        const B: &str = $B;
                        const LEN: usize = A.len() + B.len();
                        const fn combined() -> [u8; LEN] {
                            let mut out = [0u8; LEN];
                            out = copy_slice(A.as_bytes(), out, 0);
                            out = copy_slice(B.as_bytes(), out, A.len());
                            out
                        }
                        const fn copy_slice(input: &[u8], mut output: [u8; LEN], offset: usize) -> [u8; LEN] {
                            let mut index = 0;
                            loop {
                                output[offset + index] = input[index];
                                index += 1;
                                if index == input.len() {
                                    break;
                                }
                            }
                            output
                        }
                        const RESULT: &[u8] = &combined();
                        // how bad is the assumption that `&str` and `&[u8]` have the same layout?
                        const RESULT_STR: &str = unsafe { std::str::from_utf8_unchecked(RESULT) };
                        RESULT_STR
                    }};
                }

                const USES_SOURCE: &'static str = #uses;

                const VERSIONED: &'static str = combine!(#version, USES_SOURCE);

                const NEW_LINED: &'static str = combine!(VERSIONED, "\n");

                const VERTEX: &'static str = combine!(NEW_LINED, #vertex_source);
                VERTEX
            }

            fn fragment() -> &'static str {
                macro_rules! combine {
                    ($A:expr, $B:expr) => {{
                        const A: &str = $A;
                        const B: &str = $B;
                        const LEN: usize = A.len() + B.len();
                        const fn combined() -> [u8; LEN] {
                            let mut out = [0u8; LEN];
                            out = copy_slice(A.as_bytes(), out, 0);
                            out = copy_slice(B.as_bytes(), out, A.len());
                            out
                        }
                        const fn copy_slice(input: &[u8], mut output: [u8; LEN], offset: usize) -> [u8; LEN] {
                            let mut index = 0;
                            loop {
                                output[offset + index] = input[index];
                                index += 1;
                                if index == input.len() {
                                    break;
                                }
                            }
                            output
                        }
                        const RESULT: &[u8] = &combined();
                        // how bad is the assumption that `&str` and `&[u8]` have the same layout?
                        const RESULT_STR: &str = unsafe { std::str::from_utf8_unchecked(RESULT) };
                        RESULT_STR
                    }};
                }

                const USES_SOURCE: &'static str = #uses;
                const VERSIONED: &'static str = combine!(#version, USES_SOURCE);
                const NEW_LINED: &'static str = combine!(VERSIONED, "\n");

                const FRAG: &'static str = combine!(NEW_LINED, #fragment_source);
                FRAG

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

pub fn uses(info: &ShaderInfo) -> proc_macro2::TokenStream {
    let uses = info
        .uses
        .iter()
        .map(|s| {
            let idents = s.iter().map(|i| format_ident!("{}", i.to_string()));

            quote! { pub use #(#idents)::*;}
        })
        .collect::<Vec<_>>();

    quote! {
        pub mod uses {
            #(#uses)*
        }
    }
}
