use crate::{ProgramInput, shader_var::ShaderType};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
pub fn buffer_structs(input: &ProgramInput, use_crate: bool) -> proc_macro2::TokenStream {
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
            let (fields, setters): (Vec<TokenStream>, Vec<TokenStream>) = u.fields.iter().map(|f| {
                let ty = &f.t;
                let name = format_ident!("{}", f.name.to_string());

                let field = if f.is_array { quote! {#name: Vec<#ty>}} else {quote! {#name: #ty}};

                let f_name = format_ident!("{}", f.name.to_string());

                let set = quote! {
                    let size = std::mem::size_of_val(val);
                    unsafe {
                        mapping.write(val as *const _ as *const u8, size, offset);
                    };
                    offset += align;
                };

                let setter = match ty {
                    ShaderType::Primative(_) => {
                        if f.is_array {
                            quote! {
                            for item in &self.#f_name {
                                let val = item;
                                let align = attr_type_of_val(item).std430_align();
                                #set
                            }}
                        } else {
                        quote! {
                            let val = self.#f_name;
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
                                    let val = #name;
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
                use super::structs::*;

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
                        #crate_path::profiler::event!("Setting data for buffer block");
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

                impl #crate_path::SSBOBlock for #name {}
            }
        })
        .collect::<Vec<_>>();

    quote! {
        #(#structs)*
    }
}
