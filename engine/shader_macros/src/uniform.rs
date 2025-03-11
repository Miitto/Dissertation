use proc_macro::Ident;

use crate::shader_var::ShaderVar;

#[derive(Clone, Debug)]
pub enum Uniform {
    Single(SingleUniform),
    Block(UniformBlock),
}

#[derive(Clone, Debug)]
pub struct SingleUniform {
    pub var: ShaderVar,
    pub value: Option<String>,
}

#[derive(Clone, Debug)]
pub struct UniformBlock {
    pub bind: u32,
    pub name: Ident,
    pub fields: Vec<ShaderVar>,
    pub var_name: Option<Ident>,
}

impl Uniform {
    pub fn name(&self) -> &Ident {
        match self {
            Uniform::Single(s) => &s.var.name,
            Uniform::Block(b) => &b.name,
        }
    }
}
