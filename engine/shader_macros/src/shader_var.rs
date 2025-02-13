use std::fmt::Display;

use proc_macro::{Ident, Span};
use quote::{ToTokens, format_ident, quote};

#[derive(Clone, Debug)]
#[expect(dead_code)]
pub struct ShaderVar {
    pub name: Ident,
    pub t: ShaderType,
    pub type_span: Option<Span>,
}

impl Display for ShaderVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.t, self.name)
    }
}

impl ToTokens for ShaderVar {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name: proc_macro2::Ident = format_ident!("{}", self.name.to_string());
        let t = &self.t;
        tokens.extend(quote! {#t #name});
    }
}

#[derive(Clone, Debug)]
pub struct Uniform {
    pub var: ShaderVar,
    pub value: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ShaderType {
    Primative(ShaderPrimatives),
    Object(ShaderObjects),
}

#[allow(dead_code)]
impl ShaderType {
    pub fn get_member(&self, name: &Ident) -> Option<ShaderType> {
        match self {
            Self::Primative(_) => None,
            Self::Object(o) => o.get_member(name),
        }
    }
}

impl Display for ShaderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShaderType::Primative(p) => write!(f, "{}", p),
            ShaderType::Object(o) => write!(f, "{}", o),
        }
    }
}

impl ToTokens for ShaderType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ShaderType::Primative(p) => p.to_tokens(tokens),
            ShaderType::Object(o) => o.to_tokens(tokens),
        }
    }
}

impl std::ops::Mul for ShaderType {
    type Output = Result<Self, String>;
    fn mul(self, rhs: Self) -> Self::Output {
        if let Self::Object(ShaderObjects::Custom(_)) = &self {
            return Err("Can't multiply custom objects".to_string());
        }

        if let Self::Object(ShaderObjects::Custom(_)) = rhs {
            if let Self::Primative(_) = rhs {
                return Err("Can't multiply custom objects".to_string());
            }
        }

        if self == rhs {
            return Ok(self);
        }

        if let Self::Object(o) = &self {
            use ShaderObjects::*;
            use ShaderPrimatives::*;
            match o {
                Vec2 | Vec3 | Vec4 => match rhs {
                    Self::Primative(Float) | Self::Primative(Int) | Self::Primative(Double) => {
                        return Ok(self);
                    }
                    _ => {
                        return Err(format!(
                            "Invalid multiplication between {} and {}",
                            self, rhs,
                        ));
                    }
                },
                Mat2 => match rhs {
                    Self::Primative(Float) | Self::Primative(Int) | Self::Primative(Double) => {
                        return Ok(self);
                    }
                    Self::Object(Vec2) => return Ok(rhs),
                    _ => {
                        return Err(format!(
                            "Invalid multiplication between {} and {}",
                            self, rhs,
                        ));
                    }
                },
                Mat3 => match rhs {
                    Self::Primative(Float) | Self::Primative(Int) | Self::Primative(Double) => {
                        return Ok(self);
                    }
                    Self::Object(Vec3) => return Ok(rhs),
                    _ => {
                        return Err(format!(
                            "Invalid multiplication between {} and {}",
                            self, rhs,
                        ));
                    }
                },
                Mat4 => match rhs {
                    Self::Primative(Float) | Self::Primative(Int) | Self::Primative(Double) => {
                        return Ok(self);
                    }
                    Self::Object(Vec4) => return Ok(rhs),
                    _ => {
                        return Err(format!(
                            "Invalid multiplication between {} and {}",
                            self, rhs,
                        ));
                    }
                },
                _ => {}
            }
        }

        Ok(self)
    }
}

impl std::ops::Div for ShaderType {
    type Output = Result<Self, String>;

    fn div(self, _rhs: Self) -> Self::Output {
        todo!()
    }
}

impl std::ops::Add for ShaderType {
    type Output = Result<Self, String>;

    fn add(self, _rhs: Self) -> Self::Output {
        todo!()
    }
}

impl std::ops::Sub for ShaderType {
    type Output = Result<Self, String>;

    fn sub(self, _rhs: Self) -> Self::Output {
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ShaderPrimatives {
    Bool,
    Int,
    UInt,
    Float,
    Double,
}

impl ToTokens for ShaderPrimatives {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ShaderPrimatives::Bool => tokens.extend(quote! {bool}),
            ShaderPrimatives::Int => tokens.extend(quote! {i32}),
            ShaderPrimatives::UInt => tokens.extend(quote! {u32}),
            ShaderPrimatives::Float => tokens.extend(quote! {float}),
            ShaderPrimatives::Double => tokens.extend(quote! {double}),
        }
    }
}

impl Display for ShaderPrimatives {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ShaderPrimatives::Bool => "bool",
                ShaderPrimatives::Int => "int",
                ShaderPrimatives::UInt => "uint",
                ShaderPrimatives::Float => "float",
                ShaderPrimatives::Double => "double",
            }
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ShaderObjects {
    Vec2,
    Vec3,
    Vec4,
    Mat2,
    Mat3,
    Mat4,
    Custom(ShaderStruct),
}

#[allow(dead_code)]
impl ShaderObjects {
    fn vec_swizzle(len: usize, name: &str) -> Option<ShaderType> {
        let mut chars = vec!['x', 'r'];
        if len > 1 {
            chars.push('y');
            chars.push('g');
        }

        if len > 2 {
            chars.push('z');
            chars.push('b');
        }

        if len > 3 {
            chars.push('w');
            chars.push('a');
        }

        if !name.chars().all(|c| chars.contains(&c)) {
            panic!("Invalid swizzle for vec{}: {}", len, name);
        }

        Some(match len {
            1 => ShaderType::Primative(ShaderPrimatives::Float),
            2 => ShaderType::Object(ShaderObjects::Vec2),
            3 => ShaderType::Object(ShaderObjects::Vec3),
            4 => ShaderType::Object(ShaderObjects::Vec4),
            _ => return None,
        })
    }

    pub fn get_member(&self, name: &Ident) -> Option<ShaderType> {
        let name = name.to_string();
        let name = name.as_str();

        match self {
            Self::Vec2 => Self::vec_swizzle(2, name),
            Self::Vec3 => Self::vec_swizzle(3, name),
            Self::Vec4 => Self::vec_swizzle(4, name),
            // TODO: Matricies
            Self::Custom(s) => s.fields.iter().find_map(|f| {
                if f.name.to_string() == *name {
                    Some(f.t.clone())
                } else {
                    None
                }
            }),
            _ => None,
        }
    }
}

impl Display for ShaderObjects {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShaderObjects::Vec2 => write!(f, "vec2"),
            ShaderObjects::Vec3 => write!(f, "vec3"),
            ShaderObjects::Vec4 => write!(f, "vec4"),
            ShaderObjects::Mat2 => write!(f, "mat2"),
            ShaderObjects::Mat3 => write!(f, "mat3"),
            ShaderObjects::Mat4 => write!(f, "mat4"),
            ShaderObjects::Custom(s) => write!(f, "{}", s.name),
        }
    }
}

impl ToTokens for ShaderObjects {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ShaderObjects::Vec2 => tokens.extend(quote! { [f32; 2] }),
            ShaderObjects::Vec3 => tokens.extend(quote! { [f32; 3] }),
            ShaderObjects::Vec4 => tokens.extend(quote! { [f32; 4] }),
            ShaderObjects::Mat2 => tokens.extend(quote! { [[f32; 2]; 2] }),
            ShaderObjects::Mat3 => tokens.extend(quote! { [[f32; 3]; 3] }),
            ShaderObjects::Mat4 => tokens.extend(quote! { [[f32; 4]; 4] }),
            ShaderObjects::Custom(s) => {
                panic!("Can't convert glsl struct {} to a rust type", s.name);
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
