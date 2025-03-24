use std::fmt::Display;

use proc_macro::{Ident, Span};
use quote::{ToTokens, format_ident, quote};
use render_common::format::AttributeType;

#[derive(Clone, Debug)]
#[expect(dead_code)]
pub struct ShaderVar {
    pub name: Ident,
    pub t: ShaderType,
    pub type_span: Option<Span>,
    pub is_array: bool,
}

impl Display for ShaderVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}{}",
            self.t,
            self.name,
            if self.is_array { "[]" } else { "" }
        )
    }
}

impl ToTokens for ShaderVar {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name: proc_macro2::Ident = format_ident!("{}", self.name.to_string());
        let t = &self.t;
        tokens.extend(quote! {#t #name});
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ShaderType {
    Void,
    Primative(AttributeType),
    Texture(TextureType),
    Struct(ShaderStruct),
}

impl ShaderType {
    pub fn std430_align(&self) -> usize {
        match self {
            ShaderType::Primative(a) => a.std430_align(),
            ShaderType::Texture(_) => todo!("Texture in interface block"),
            ShaderType::Struct(s) => s.std430_align(),
            ShaderType::Void => 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TextureType {
    Image2D,
}

fn get_primative(val: &str) -> Option<AttributeType> {
    Some(match val {
        "byte" => AttributeType::I8,
        "bool" => AttributeType::Bool,
        "int" => AttributeType::I32,
        "uint" => AttributeType::U32,
        "float" => AttributeType::F32,
        "vec2" => AttributeType::F32F32,
        "vec3" => AttributeType::F32F32F32,
        "vec4" => AttributeType::F32F32F32F32,
        "mat4" => AttributeType::F32x4x4,
        "ivec2" => AttributeType::I32I32,
        "ivec3" => AttributeType::I32I32I32,
        "ivec4" => AttributeType::I32I32I32I32,
        "uvec2" => AttributeType::U32U32,
        "uvec3" => AttributeType::U32U32U32,
        _ => return None,
    })
}

fn get_texture(val: &str) -> Option<TextureType> {
    Some(match val {
        "image2D" => TextureType::Image2D,
        _ => {
            return None;
        }
    })
}

impl ShaderType {
    pub fn from(val: &str, structs: &[ShaderStruct]) -> Option<Self> {
        if val == "void" {
            Some(ShaderType::Void)
        } else if let Some(s) = structs.iter().find(|s| s.name.to_string() == val) {
            Some(ShaderType::Struct(s.clone()))
        } else if let Some(t) = get_primative(val) {
            Some(ShaderType::Primative(t))
        } else {
            get_texture(val).map(ShaderType::Texture)
        }
    }
}

impl Display for ShaderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShaderType::Void => write!(f, "void"),
            ShaderType::Primative(t) => {
                use AttributeType::*;
                write!(
                    f,
                    "{}",
                    match t {
                        I8 => "byte",
                        Bool => "bool",
                        I32 => "int",
                        U32 => "uint",
                        F32 => "float",
                        F64 => "double",
                        F32F32 => "vec2",
                        F32F32F32 => "vec3",
                        F32F32F32F32 => "vec4",
                        I32I32 => "ivec2",
                        I32I32I32 => "ivec3",
                        I32I32I32I32 => "ivec4",
                        F32x4x4 => "mat4",
                        U32U32 => "uvec2",
                        U32U32U32 => "uvec3",
                        _ => todo!("Convert {:?} to GLSL", t),
                    }
                )
            }
            ShaderType::Texture(t) => match t {
                TextureType::Image2D => write!(f, "image2D"),
            },
            ShaderType::Struct(s) => write!(f, "{}", s.name),
        }
    }
}

impl ToTokens for ShaderType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Primative(t) => {
                use AttributeType::*;
                match t {
                    I8 => tokens.extend(quote! {i8}),
                    I32 => tokens.extend(quote! {i32}),
                    U32 => tokens.extend(quote! {u32}),
                    F32 => tokens.extend(quote! {f32}),
                    F32F32 => tokens.extend(quote! {[f32; 2]}),
                    F32F32F32 => tokens.extend(quote! {[f32; 3]}),
                    F32F32F32F32 => tokens.extend(quote! {[f32; 4]}),
                    I32I32 => tokens.extend(quote! {[i32; 2]}),
                    I32I32I32 => tokens.extend(quote! {[i32; 3]}),
                    I32I32I32I32 => tokens.extend(quote! {[i32; 4]}),
                    F32x4x4 => tokens.extend(quote! {[[f32; 4]; 4]}),
                    U32U32 => tokens.extend(quote! {[u32; 2]}),
                    _ => todo!("Convert {:?} to rust type", t),
                }
            }
            Self::Void => tokens.extend(quote! {()}),
            Self::Texture(t) => match t {
                TextureType::Image2D => tokens.extend(quote! {renderer::texture::Texture2D}),
            },
            Self::Struct(s) => {
                let name = format_ident!("{}", s.name.to_string());
                tokens.extend(quote! {#name})
            }
        }
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(crate) struct ShaderStruct {
    pub name: Ident,
    pub fields: Vec<ShaderVar>,
}

impl ShaderStruct {
    pub fn std430_align(&self) -> usize {
        self.fields.iter().map(|f| f.t.std430_align()).sum()
    }
}

impl PartialEq for ShaderStruct {
    fn eq(&self, other: &Self) -> bool {
        self.name.to_string() == other.name.to_string()
    }
}

impl Display for ShaderStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "struct {} {{\n{}\n}};",
            self.name,
            self.fields
                .iter()
                .map(|f| format!("    {};", f))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

impl ToTokens for ShaderStruct {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = format_ident!("{}", self.name.to_string());

        let fields = self.fields.iter().map(|f| {
            let name = format_ident!("{}", f.name.to_string());
            let t = &f.t;
            if f.is_array {
                quote! {pub #name: Vec<#t>}
            } else {
                quote! {pub #name: #t}
            }
        });

        tokens.extend(quote! {
            struct #name {
                #(#fields),*
            }
        });
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ShaderFunction {
    pub var: ShaderVar,
    pub params: Vec<ShaderVar>,
    pub content: String,
}

impl Display for ShaderFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params = self
            .params
            .iter()
            .map(|p| format!("{}", p))
            .collect::<Vec<String>>()
            .join(", ");

        write!(
            f,
            "{} {}({}) {{\n{}\n}}",
            self.var.t, self.var.name, params, self.content
        )
    }
}
