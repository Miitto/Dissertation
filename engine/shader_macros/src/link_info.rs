use std::fmt::{Display, Formatter};

use crate::{
    parse_glsl::{ShaderInfo, ShaderStruct},
    shader_var::ShaderVarType,
};

#[derive(Clone, Debug)]
pub(crate) struct LinkedShaderVar {
    pub t: ShaderReturnType,
    pub name: String,
}

#[derive(Clone, Debug)]
pub(crate) enum ShaderReturnType {
    Primative(ShaderVarType),
    Struct(ShaderStruct),
    Unknown(String),
}

impl ShaderReturnType {
    pub fn get_struct(&self) -> &ShaderStruct {
        match self {
            ShaderReturnType::Struct(s) => s,
            _ => panic!("Expected struct"),
        }
    }
}

impl quote::ToTokens for ShaderReturnType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ShaderReturnType::Primative(t) => t.to_tokens(tokens),
            _ => panic!("Expected primative"),
        }
    }
}

impl Display for ShaderReturnType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ShaderReturnType::Primative(t) => write!(f, "{}", t.to_glsl()),
            ShaderReturnType::Struct(s) => write!(f, "{}", s.name),
            ShaderReturnType::Unknown(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LinkedShaderFunction {
    pub return_type: ShaderReturnType,
    pub name: String,
    pub params: Vec<LinkedShaderVar>,
    pub content: String,
}

impl Display for LinkedShaderFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let params = self
            .params
            .iter()
            .map(|p| format!("{} {}", p.t, p.name))
            .collect::<Vec<String>>()
            .join(", ");
        write!(
            f,
            "{} {}({}) {{\n{}\n}}",
            self.return_type, self.name, params, self.content
        )
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LinkedShaderInfo {
    pub structs: Vec<ShaderStruct>,
    pub functions: Vec<LinkedShaderFunction>,
    pub uniforms: Vec<LinkedShaderVar>,
    pub vertex_fn: Option<LinkedShaderFunction>,
    pub frag_fn: Option<LinkedShaderFunction>,
    pub geometry_fn: Option<LinkedShaderFunction>,
}

pub(crate) fn link_info(info: ShaderInfo) -> LinkedShaderInfo {
    // Link Functions
    let functions: Vec<LinkedShaderFunction> = info
        .functions
        .into_iter()
        .map(|f| {
            let return_type = match &f.return_type {
                ShaderVarType::Other(name) => {
                    let struct_type = info.structs.iter().find(|s| s.name == *name);
                    match struct_type {
                        Some(struct_type) => ShaderReturnType::Struct(struct_type.clone()),
                        None => ShaderReturnType::Unknown(name.clone()),
                    }
                }
                _ => ShaderReturnType::Primative(f.return_type.clone()),
            };

            let params = f
                .params
                .into_iter()
                .map(|p| {
                    let return_type = match &p.r#type {
                        ShaderVarType::Other(name) => {
                            let struct_type = info.structs.iter().find(|s| s.name == *name);
                            match struct_type {
                                Some(struct_type) => ShaderReturnType::Struct(struct_type.clone()),
                                None => ShaderReturnType::Unknown(name.clone()),
                            }
                        }
                        _ => ShaderReturnType::Primative(p.r#type),
                    };

                    LinkedShaderVar {
                        t: return_type,
                        name: p.name,
                    }
                })
                .collect();

            LinkedShaderFunction {
                return_type,
                name: f.name,
                params,
                content: f.content,
            }
        })
        .collect();

    let uniforms = info
        .uniforms
        .into_iter()
        .map(|u| {
            let t = match u.r#type {
                ShaderVarType::Other(name) => {
                    let struct_type = info.structs.iter().find(|s| s.name == name);
                    match struct_type {
                        Some(struct_type) => ShaderReturnType::Struct(struct_type.clone()),
                        None => ShaderReturnType::Unknown(name),
                    }
                }
                _ => ShaderReturnType::Primative(u.r#type.clone()),
            };

            LinkedShaderVar { t, name: u.name }
        })
        .collect();

    let vertex_fn = if let Some(vertex_fn_name) = &info.vertex_fn {
        functions
            .iter()
            .find(|f| f.name == *vertex_fn_name)
            .cloned()
    } else {
        None
    };
    let frag_fn = if let Some(frag_fn_name) = &info.frag_fn {
        functions.iter().find(|f| f.name == *frag_fn_name).cloned()
    } else {
        None
    };
    let geometry_fn = if let Some(geometry_fn_name) = &info.geometry_fn {
        functions
            .iter()
            .find(|f| f.name == *geometry_fn_name)
            .cloned()
    } else {
        None
    };

    let functions = functions
        .into_iter()
        .filter(|func| {
            let v = info.vertex_fn.as_ref().is_some_and(|n| &func.name != n);
            let f = info.frag_fn.as_ref().is_some_and(|n| &func.name != n);
            let g = info.geometry_fn.as_ref().is_some_and(|n| &func.name != n);

            v && f && g
        })
        .collect();

    LinkedShaderInfo {
        structs: info.structs,
        functions,
        uniforms,
        vertex_fn,
        frag_fn,
        geometry_fn,
    }
}
