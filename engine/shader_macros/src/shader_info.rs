use proc_macro::{Ident, Span, TokenTree};

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

impl ShaderInfo {
    pub fn get_type(&self, type_name: &TokenTree, local_vars: &[ShaderVar]) -> ShaderType {
        match type_name {
            TokenTree::Literal(lit) => {
                let lit = lit.to_string();

                if lit.parse::<i32>().is_ok() {
                    return ShaderType::Primative(ShaderPrimatives::Int);
                }

                if lit.chars().last().is_some_and(|c| c == 'f') || lit.parse::<f32>().is_ok() {
                    return ShaderType::Primative(ShaderPrimatives::Float);
                }
            }
            TokenTree::Ident(ident) => {
                if let Some(var) = self.get_var(ident, local_vars) {
                    return var.t;
                }

                let type_name = ident.to_string();
                let primative = match type_name.as_str() {
                    "int" => Some(ShaderType::Primative(ShaderPrimatives::Int)),
                    "uint" => Some(ShaderType::Primative(ShaderPrimatives::UInt)),
                    "float" => Some(ShaderType::Primative(ShaderPrimatives::Float)),
                    "double" => Some(ShaderType::Primative(ShaderPrimatives::Double)),
                    "bool" => Some(ShaderType::Primative(ShaderPrimatives::Bool)),
                    _ => None,
                };

                if let Some(p) = primative {
                    return p;
                }

                let object = match type_name.as_str() {
                    "vec2" => Some(ShaderType::Object(ShaderObjects::Vec2)),
                    "vec3" => Some(ShaderType::Object(ShaderObjects::Vec3)),
                    "vec4" => Some(ShaderType::Object(ShaderObjects::Vec4)),
                    "mat2" => Some(ShaderType::Object(ShaderObjects::Mat2)),
                    "mat3" => Some(ShaderType::Object(ShaderObjects::Mat3)),
                    "mat4" => Some(ShaderType::Object(ShaderObjects::Mat4)),
                    _ => None,
                };

                if let Some(o) = object {
                    return o;
                }

                for s in &self.structs {
                    if s.name == *type_name {
                        return ShaderType::Object(ShaderObjects::Custom(s.clone()));
                    }
                }
            }
            _ => {}
        }

        ShaderType::Unknown(type_name.to_string())
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
