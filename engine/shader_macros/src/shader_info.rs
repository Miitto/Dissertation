use proc_macro::{Diagnostic, Ident, Level, Span};
use render_common::format::AttributeType;

use crate::{
    shader_var::{ShaderFunction, ShaderStruct, ShaderType, ShaderVar},
    uniform::Uniform,
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
    pub fn get_type(&self, ident: &Ident) -> Result<ShaderType, Diagnostic> {
        if let Some(s) = ShaderType::from(ident.to_string().as_str(), &self.structs) {
            return Ok(s);
        }

        Err(Diagnostic::spanned(
            ident.span(),
            Level::Error,
            format!("Unknown type: {}", ident),
        ))
    }
}
