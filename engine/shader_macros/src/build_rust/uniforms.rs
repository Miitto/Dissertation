use crate::{ProgramInput, shader_var::ShaderType, uniform::Uniform};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::buffer_structs;

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
            let (fields, setters): (Vec<TokenStream>, Vec<TokenStream>) = u.fields.iter().map(|f| {
                let ty = &f.t;
                let name = format_ident!("{}", f.name.to_string());

                let field = if f.is_array { quote! {#name: Vec<#ty>}} else {quote! {#name: #ty}};

                let f_name = format_ident!("{}", f.name.to_string());

                let set = quote! {
                    let size = std::mem::size_of_val(&val);
                    unsafe {
                        mapping.write(val.as_ptr() as *const u8, size, offset);
                    };
                    offset += align;
                };

                let setter = match ty {
                    ShaderType::Primative(_) => {
                        if f.is_array {
                            quote! {
                            for item in &self.#f_name {
                                let val = [item];
                                let align = attr_type_of_val(item).std430_align();
                                #set
                            }}
                        } else {
                        quote! {
                            let val = [self.#f_name];
                            let align = attr_type_of_val(&self.#f_name).std430_align();
                            #set
                        }}
                    },
                    ShaderType::Struct(s) => {
                        let setters = s.fields.iter().map(|s| {
                            let s_name = format_ident!("{}", s.name.to_string());
                            let name = if f.is_array { quote! {item}} else {quote! {self.#f_name.#s_name}};
                            let align = if f.is_array { quote! {attr_type_of_val(&item.#s_name).std430_align()}} else {quote! {attr_type_of_val(&self.#f_name.#s_name).std430_align()}};
                               quote! {
                                    let val = [#name];
                                    let align = #align;
                                    #set
                               }
                        }).collect::<Vec<_>>();


                        if f.is_array {
                            quote! {
                                for item in &self.#f_name {
                                    #(#setters)*
                                }
                            }
                        } else {quote! {
                            #(#setters)*
                        }}
                    }, _ => {
                        panic!("Unsupported type in buffer block: {:?}", ty);
                    }
                };

                (field, setter)
            }).unzip();

            let bind = u.bind;

            let size = if !u.fields.iter().any(|f| f.is_array) {let size = u.fields.iter().map(|f| f.t.std430_align()).sum::<usize>(); quote!{#size}} else {
                let static_size = u.fields.iter().filter(|f| !f.is_array).map(|f| f.t.std430_align()).sum::<usize>();

                let calc_size = u.fields.iter().filter(|f| f.is_array).map(|f| {
                    let f_name = format_ident!("{}", f.name.to_string());
                    let field_size = f.t.std430_align();
                    quote! {
                        let len = self.#f_name.len();
                        size += #field_size * len;
                    }
                });

                quote! {
                    let mut size = 0;

                    #(#calc_size)*

                    #static_size + size
                }
            };

            quote! {
                #[derive(Debug)]
                pub struct #name {
                    #(pub #fields),*
                }

                impl #name {
                    const BIND: u32 = #bind;
                }

                impl #crate_path::LayoutBlock for #name {
                    fn bind_point() -> u32 {
                        Self::BIND
                    }

                    fn size(&self) -> usize {
                        #size
                    }

                    fn set_buffer_data<'a, B: #crate_path::buffers::RawBuffer>(&self, mapping: &mut #crate_path::buffers::Mapping<'a, B>, offset: usize) -> Result<usize, #crate_path::buffers::BufferError> {
                        #crate_path::profiler::event!("Setting data for uniform block");
                        let mut offset = offset;

                        const fn attr_type_of_val<T: #crate_path::vertex::Attribute>(_: &T)
                                -> #crate_path::vertex::format::AttributeType
                            {
                                <T as #crate_path::vertex::Attribute>::TYPE
                            }

                        #(#setters)*

                        Ok(offset)
                    }
                }

                impl #crate_path::UniformBlock for #name {}
            }
        })
        .collect::<Vec<_>>();

    quote! {
        #(#structs)*
    }
}
