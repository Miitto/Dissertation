use proc_macro2::Span;
use syn::{Ident, parse::ParseStream};

use crate::shader_var::{
    ShaderFunction, ShaderObjects, ShaderPrimatives, ShaderStruct, ShaderType, ShaderVar,
};

#[derive(Clone, Debug, Default)]
pub(crate) struct ShaderInfo {
    pub structs: Vec<ShaderStruct>,
    pub functions: Vec<ShaderFunction>,
    pub uniforms: Vec<ShaderVar>,
    pub vertex_fn: Option<ShaderFunction>,
    pub frag_fn: Option<ShaderFunction>,
    pub geometry_fn: Option<ShaderFunction>,
}

impl syn::parse::Parse for ShaderInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        crate::parse::glsl::parse_glsl(input)
    }
}

impl ShaderInfo {
    pub fn get_type(
        &self,
        ident: &Ident,
        local_vars: &[ShaderVar],
        use_vars: bool,
    ) -> syn::Result<ShaderType> {
        if use_vars {
            if let Some(var) = self.get_var(ident, local_vars) {
                return Ok(var.t);
            }
        }

        let primative = match ident.to_string().as_str() {
            "int" => Some(ShaderType::Primative(ShaderPrimatives::Int)),
            "uint" => Some(ShaderType::Primative(ShaderPrimatives::UInt)),
            "float" => Some(ShaderType::Primative(ShaderPrimatives::Float)),
            "double" => Some(ShaderType::Primative(ShaderPrimatives::Double)),
            "bool" => Some(ShaderType::Primative(ShaderPrimatives::Bool)),
            _ => None,
        };

        if let Some(primative) = primative {
            return Ok(primative);
        }

        let object = match ident.to_string().as_str() {
            "vec2" => Some(ShaderType::Object(ShaderObjects::Vec2)),
            "vec3" => Some(ShaderType::Object(ShaderObjects::Vec3)),
            "vec4" => Some(ShaderType::Object(ShaderObjects::Vec4)),
            "mat2" => Some(ShaderType::Object(ShaderObjects::Mat2)),
            "mat3" => Some(ShaderType::Object(ShaderObjects::Mat3)),
            "mat4" => Some(ShaderType::Object(ShaderObjects::Mat4)),
            _ => None,
        };

        if let Some(object) = object {
            return Ok(object);
        }

        for s in &self.structs {
            if s.name == *ident.to_string() {
                return Ok(ShaderType::Object(ShaderObjects::Custom(s.clone())));
            }
        }

        Err(syn::Error::new(
            ident.span(),
            format!("Unknown type: {}", ident),
        ))
    }

    pub fn get_var(&self, name: &Ident, local_vars: &[ShaderVar]) -> Option<ShaderVar> {
        let name = name.to_string();
        if let Some(var) = local_vars.iter().find(|v| v.name == name) {
            return Some(var.clone());
        }

        for uniform in &self.uniforms {
            if uniform.name == name {
                return Some(uniform.clone());
            }
        }

        // TODO: Builtin vars
        if let Some(name) = name.strip_prefix("gl_") {
            return Some(match name {
                "Position" => ShaderVar {
                    name: "gl_Position".to_string(),
                    t: ShaderType::Object(ShaderObjects::Vec4),
                    name_span: Span::call_site(),
                    type_span: None,
                },
                _ => {
                    return None;
                }
            });
        }
        None
    }
}
