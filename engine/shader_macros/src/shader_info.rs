use proc_macro::{Diagnostic, Ident, Level, Span};

use crate::shader_var::{
    ShaderFunction, ShaderObjects, ShaderPrimatives, ShaderStruct, ShaderType, ShaderVar, Uniform,
};

#[derive(Clone, Debug, Default)]
pub(crate) struct ShaderInfo {
    pub structs: Vec<ShaderStruct>,
    pub functions: Vec<ShaderFunction>,
    pub uniforms: Vec<Uniform>,
    pub vertex_fn: Option<ShaderFunction>,
    pub frag_fn: Option<ShaderFunction>,
    pub geometry_fn: Option<ShaderFunction>,
    pub includes: Vec<String>,
}

impl ShaderInfo {
    pub fn get_type(
        &self,
        ident: &Ident,
        local_vars: &[ShaderVar],
        use_vars: bool,
    ) -> Result<ShaderType, Diagnostic> {
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
            if s.name.to_string() == *ident.to_string() {
                return Ok(ShaderType::Object(ShaderObjects::Custom(s.clone())));
            }
        }

        Err(Diagnostic::spanned(
            ident.span(),
            Level::Error,
            format!("Unknown type: {}", ident),
        ))
    }

    pub fn get_var(&self, name: &Ident, local_vars: &[ShaderVar]) -> Option<ShaderVar> {
        let name = name.to_string();
        if let Some(var) = local_vars.iter().find(|v| v.name.to_string() == name) {
            return Some(var.clone());
        }

        for uniform in &self.uniforms {
            if uniform.var.name.to_string() == name {
                return Some(uniform.var.clone());
            }
        }

        // TODO: Builtin vars
        if let Some(name) = name.strip_prefix("gl_") {
            return Some(match name {
                "Position" => ShaderVar {
                    name: Ident::new("gl_Position", Span::call_site()),
                    t: ShaderType::Object(ShaderObjects::Vec4),
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
