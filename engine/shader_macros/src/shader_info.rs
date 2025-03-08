use proc_macro::{Diagnostic, Ident, Level, Span};
use render_common::format::AttributeType;

use crate::shader_var::{ShaderFunction, ShaderStruct, ShaderType, ShaderVar, Uniform};

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

        if let Some(s) = ShaderType::from(ident.to_string().as_str(), &self.structs) {
            return Ok(s);
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
                    t: ShaderType::Primative(AttributeType::F32F32F32F32),
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
