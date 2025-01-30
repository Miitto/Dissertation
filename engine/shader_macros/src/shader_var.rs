use proc_macro::Span;
use quote::{format_ident, quote};

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug, PartialEq)]
pub enum ShaderVarType {
    Int,
    Float,
    Long,
    Double,
    Vec2,
    Vec3,
    Vec4,
    Other(String),
}
impl ShaderVarType {
    pub(crate) fn is_type(as_str: &str) -> bool {
        let converted: ShaderVarType = as_str.into();
        !matches!(converted, ShaderVarType::Other(_))
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

        let long = value.strip_suffix('l');
        if let Some(long) = long {
            if long.parse::<i64>().is_ok() {
                return Self::Long;
            }
        }
        let upper_long = value.strip_suffix('L');
        if let Some(upper_long) = upper_long {
            if upper_long.parse::<i64>().is_ok() {
                return Self::Long;
            }
        }

        if value.parse::<i32>().is_ok() {
            return Self::Int;
        }

        if value.parse::<f32>().is_ok() {
            return Self::Long;
        }

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
