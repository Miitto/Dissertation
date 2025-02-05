use std::fmt::Display;

use proc_macro::Span;
use quote::{format_ident, quote};

#[derive(Clone, Debug)]
#[expect(dead_code)]
pub struct ShaderVar {
    pub name: String,
    pub r#type: ShaderVarType,
    pub name_span: Span,
    pub type_span: Option<Span>,
}

impl ShaderVar {
    pub fn new(
        r#type: ShaderVarType,
        type_span: Option<Span>,
        name: String,
        name_span: Span,
    ) -> Self {
        Self {
            name,
            r#type,
            type_span,
            name_span,
        }
    }
}

impl Display for ShaderVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.r#type.to_glsl(), self.name)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ShaderVarType {
    Bool,
    Int,
    Uint,
    Float,
    Double,
    Vec2,
    Vec3,
    Vec4,
    Mat2,
    Mat3,
    Mat4,
    DMat4,
    Other(String),
}
impl ShaderVarType {
    pub(crate) fn to_glsl(&self) -> String {
        use ShaderVarType::*;
        match self {
            Bool => "bool".to_string(),
            Int => "int".to_string(),
            Uint => "uint".to_string(),
            Float => "float".to_string(),
            Double => "double".to_string(),
            Vec2 => "vec2".to_string(),
            Vec3 => "vec3".to_string(),
            Vec4 => "vec4".to_string(),
            Mat2 => "mat2".to_string(),
            Mat3 => "mat3".to_string(),
            Mat4 => "mat4".to_string(),
            DMat4 => "dmat4".to_string(),
            Other(s) => s.clone(),
        }
    }
}

impl From<&str> for ShaderVarType {
    fn from(value: &str) -> Self {
        let float = value.strip_suffix('f');
        if let Some(float) = float {
            if float.parse::<f32>().is_ok() {
                return Self::Float;
            }
        }

        if value.parse::<i32>().is_ok() {
            return Self::Int;
        }

        if value.parse::<f32>().is_ok() {
            return Self::Float;
        }

        match value {
            "bool" => Self::Bool,
            "int" => Self::Int,
            "uint" => Self::Uint,
            "float" => Self::Float,
            "double" => Self::Double,
            "vec2" => Self::Vec2,
            "vec3" => Self::Vec3,
            "vec4" => Self::Vec4,
            "mat2" => Self::Mat2,
            "mat3" => Self::Mat3,
            "mat4" => Self::Mat4,
            "dmat4" => Self::DMat4,
            _ => Self::Other(value.to_string()),
        }
    }
}

impl From<&String> for ShaderVarType {
    fn from(value: &String) -> Self {
        value.as_str().into()
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
            Bool => quote! { bool },
            Int => quote! { i32 },
            Uint => quote! { u32 },
            Float => quote! { f32 },
            Double => quote! { f64 },
            Vec2 => quote! { [f32; 2] },
            Vec3 => quote! { [f32; 3] },
            Vec4 => quote! { [f32; 4] },
            Mat2 => quote! { [[f32; 2]; 2] },
            Mat3 => quote! { [[f32; 3]; 3] },
            Mat4 => quote! { [[f32; 4]; 4] },
            DMat4 => quote! { [[f64; 4]; 4] },
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
