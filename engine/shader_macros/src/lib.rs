use proc_macro::{Group, Ident, Literal, Span, TokenStream, TokenTree};
use proc_macro2::{Punct, Spacing};
use quote::{TokenStreamExt, format_ident, quote};
use shaders_common::ShaderType;

mod parse_glsl;

struct ShaderMeta {
    t: ShaderType,
    name: String,
    version: i32,
}

struct ShaderInput {
    meta: ShaderMeta,
    content: String,
    shader_in: Vec<ShaderVar>,
    shader_out: Vec<ShaderVar>,
    shader_uniforms: Vec<ShaderVar>,
}

fn parse_meta(input: &mut impl Iterator<Item = TokenTree>) -> (ShaderMeta, Group) {
    let shader_type = input.next().expect("Shader needs have a type");
    _ = input.next().expect("Missing comma");
    let shader_name = input
        .next()
        .expect("Shader needs to have a name")
        .to_string();
    _ = input.next().expect("Missing comma");
    let mut shader_version: Option<i32> = None;
    let mut shader_content = None;

    match input.next() {
        Some(item) => match item {
            TokenTree::Group(g) => {
                shader_content = Some(g);
            }
            TokenTree::Literal(l) => {
                shader_version = Some(
                    l.to_string()
                        .parse()
                        .expect("Failed to parse shader version"),
                );
            }
            _ => {
                panic!("Invalid shader content");
            }
        },
        None => {
            panic!("Shader is missing its content");
        }
    }

    if shader_content.is_none() {
        input.next();
        shader_content = input.next().and_then(|item| match item {
            TokenTree::Group(g) => Some(g),
            _ => None,
        });
    }

    let shader_type: ShaderType = shader_type
        .to_string()
        .try_into()
        .expect("Invalid Shader Type");

    (
        ShaderMeta {
            t: shader_type,
            name: shader_name,
            version: shader_version.unwrap_or(330),
        },
        shader_content.expect("Shader is missing its content"),
    )
}

#[proc_macro]
pub fn shader(input: TokenStream) -> TokenStream {
    let mut iter = input.into_iter();

    let (meta, content) = parse_meta(&mut iter);

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

    let native = TokenStream::from(expanded);

    let span = native.to_string();

    dbg!(span);

    native
}

struct ShaderVar {
    name: String,
    r#type: ShaderVarType,
}

impl ShaderVar {
    pub fn new(r#type: ShaderVarType, name: String) -> Self {
        Self { name, r#type }
    }
}

enum ShaderVarType {
    Int,
    Float,
    Long,
    Double,
    Vec2,
    Vec3,
    Vec4,
    Other(String),
}

impl From<&str> for ShaderVarType {
    fn from(value: &str) -> Self {
        match value {
            "int" => Self::Int,
            "float" => Self::Float,
            "long" => Self::Long,
            "double" => Self::Double,
            "vec2" => Self::Vec2,
            "vec3" => Self::Vec3,
            "vec4" => Self::Vec4,
            _ => Self::Other(value.to_string()),
        }
    }
}

impl From<String> for ShaderVarType {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

impl quote::ToTokens for ShaderVarType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use ShaderVarType::*;
        let t = match self {
            Int => quote! { i32 },
            Long => quote! { i64 },
            Float => quote! { f32 },
            Double => quote! { f64 },
            Vec2 => quote! { [f32; 2] },
            Vec3 => quote! { [f32; 3] },
            Vec4 => quote! { [f32; 4] },
            Other(s) => {
                let ident = format_ident!("{}", s);
                quote! {#ident}
            }
        };

        tokens.extend(t);
    }
}

impl quote::ToTokens for ShaderVar {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = format_ident!("{}", &self.name);
        let t = &self.r#type;
        let t = quote! {
            #name: #t,
        };

        tokens.extend(t);
    }
}
