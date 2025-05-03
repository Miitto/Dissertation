use proc_macro::{Diagnostic, Ident, Level};

use crate::{
    shader_var::{ShaderConstant, ShaderFunction, ShaderStruct, ShaderType},
    uniform::{LayoutBlock, Uniform},
};

#[derive(Clone, Debug, Default)]
pub(crate) struct ShaderInfo {
    pub structs: Vec<ShaderStruct>,
    pub functions: Vec<ShaderFunction>,
    pub uniforms: Vec<Uniform>,
    pub buffers: Vec<LayoutBlock>,
    pub vertex_fn: Option<ShaderFunction>,
    pub frag_fn: Option<ShaderFunction>,
    pub geometry_fn: Option<ShaderFunction>,
    pub includes: Vec<String>,
    pub uses: Vec<Vec<Ident>>,
    pub compute: Vec<ComputeInfo>,
    pub constants: Vec<ShaderConstant>,
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

#[derive(Clone, Debug)]
pub(crate) struct ComputeInfo {
    pub name: Ident,
    pub size: (u32, u32, u32),
    pub function: ShaderFunction,
}
