use proc_macro::TokenStream;
use quote::{format_ident, quote};
use shader_var::ShaderVar;
use shaders_common::ShaderType;

mod parse_glsl;
mod parse_meta;
mod shader_var;

struct ShaderMeta {
    t: ShaderType,
    name: String,
    version: i32,
}

#[allow(dead_code)]
struct ShaderInput {
    meta: ShaderMeta,
    content: String,
    shader_in: Vec<ShaderVar>,
    shader_out: Vec<ShaderVar>,
    shader_uniforms: Vec<ShaderVar>,
}

#[proc_macro]
pub fn shader(input: TokenStream) -> TokenStream {
    let mut iter = input.into_iter();

    let (meta, content) = parse_meta::parse_meta(&mut iter);

    let shader = parse_glsl::parse_glsl(content, meta);

    let shader_name = format_ident!("{}{}", shader.meta.name, shader.meta.t.to_string());
    let shader_in = shader.shader_in;

    let content = shader.content;

    let vertex_impl = if let ShaderType::Vertex = shader.meta.t {
        let shader_in_names = shader_in.iter().map(|s| format_ident!("{}", s.name));
        quote! {
            ::glium::implement_vertex!(#shader_name #(, #shader_in_names)*);
        }
    } else {
        quote! {}
    };

    let source = proc_macro2::Literal::string(&content);

    let expanded = quote! {
        #[derive(Debug, Copy, Clone)]
        struct #shader_name {
            #(pub #shader_in),*
        }

        #vertex_impl

        impl #shader_name {
            pub fn source() -> &'static str {
                #source
            }
        }
    };

    TokenStream::from(expanded)
}
