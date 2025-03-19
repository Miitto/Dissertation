use proc_macro::Ident;

use crate::shader_var::ShaderVar;

#[derive(Clone, Debug)]
pub enum Uniform {
    Single(SingleUniform),
    Texture(TextureUniform),
    Block(LayoutBlock),
}

#[derive(Clone, Debug)]
pub struct SingleUniform {
    pub var: ShaderVar,
    pub value: Option<String>,
}

#[derive(Clone, Debug)]
pub struct TextureUniform {
    pub bind: u32,
    pub var: ShaderVar,
}

#[derive(Clone, Debug)]
pub struct LayoutBlock {
    pub bind: u32,
    pub name: Ident,
    pub fields: Vec<ShaderVar>,
    pub var_name: Option<Ident>,
    pub is_array: bool,
}

#[expect(dead_code)]
impl Uniform {
    pub fn name(&self) -> &Ident {
        match self {
            Uniform::Single(s) => &s.var.name,
            Uniform::Texture(t) => &t.var.name,
            Uniform::Block(b) => &b.name,
        }
    }
}
