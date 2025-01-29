use quote::{format_ident, quote};

pub struct ShaderVar {
    pub name: String,
    pub r#type: ShaderVarType,
}

impl ShaderVar {
    pub fn new(r#type: ShaderVarType, name: String) -> Self {
        Self { name, r#type }
    }
}

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
