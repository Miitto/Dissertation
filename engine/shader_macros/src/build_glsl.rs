use proc_macro::{Diagnostic, Level};
use std::fmt::Write;

use crate::{ProgramInput, ShaderInfo, shader_var::ShaderType, uniform::Uniform};

fn get_uniforms(info: &ShaderInfo) -> String {
    info.uniforms
        .iter()
        .map(|uniform| match uniform {
            Uniform::Single(s) => {
                let default_value = if let Some(v) = s.value.as_ref() {
                    format!(" = {}", v)
                } else {
                    "".to_string()
                };
                format!("uniform {} {}{};", s.var.t, s.var.name, default_value)
            }
            Uniform::Block(b) => {
                let fields = b.fields.iter().fold(String::default(), |mut s, f| {
                    _ = writeln!(s, "\t{};", f);
                    s
                });

                format!(
                    "layout(std140, binding = {}) uniform {} {{\n{}}} {};",
                    b.bind,
                    b.name,
                    fields,
                    b.var_name
                        .as_ref()
                        .map(|i| i.to_string())
                        .unwrap_or_default()
                )
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn get_buffers(info: &ShaderInfo) -> String {
    info.buffers
        .iter()
        .map(|b| {
            let fields = b.fields.iter().fold(String::default(), |mut s, f| {
                _ = writeln!(s, "\t{};", f);
                s
            });

            format!(
                "layout(std430, binding = {}) uniform {} {{\n{}}} {};",
                b.bind,
                b.name,
                fields,
                b.var_name
                    .as_ref()
                    .map(|i| i.to_string())
                    .unwrap_or_default()
            )
        })
        .collect()
}

fn get_structs(info: &ShaderInfo) -> String {
    info.structs
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
        .join("\n\n")
}

fn get_functions(info: &ShaderInfo) -> String {
    info.functions
        .iter()
        .map(|f| f.to_string())
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn vertex_shader(ProgramInput { content: info, .. }: &ProgramInput) -> String {
    let uniforms = get_uniforms(info);

    let structs = get_structs(info);

    let functions = get_functions(info);

    let buffers = get_buffers(info);

    let vertex_fn = if let Some(vertex_fn) = info.vertex_fn.as_ref() {
        vertex_fn
    } else {
        Diagnostic::new(
            Level::Error,
            "No vertex function found when building vertex shader",
        )
        .emit();
        return "".to_string();
    };

    let vertex_input = match &vertex_fn.params[0].t {
        ShaderType::Struct(s) => s,
        _ => {
            panic!("Fragment function must take a struct as input");
        }
    };

    let in_vars = {
        let iter = vertex_input.fields.iter();

        if let Some(instance) = vertex_fn.params.get(1).map(|i| match &i.t {
            ShaderType::Struct(s) => s,
            _ => {
                panic!("Vertex function must take a struct as input");
            }
        }) {
            let iter = iter.chain(instance.fields.iter());
            iter.enumerate()
                .map(|(idx, f)| format!("layout(location = {}) in {} {};", idx, f.t, f.name))
                .collect::<Vec<String>>()
                .join("\n")
        } else {
            iter.enumerate()
                .map(|(idx, f)| format!("layout(location = {}) in {} {};", idx, f.t, f.name))
                .collect::<Vec<String>>()
                .join("\n")
        }
    };

    let in_to_struct_assign = vertex_input
        .fields
        .iter()
        .map(|f| format!("    vertex_input.{} = {};", f.name, f.name))
        .collect::<Vec<String>>()
        .join("\n");

    let in_to_struct = format!(
        "{} vertex_input;\n{}",
        vertex_input.name, in_to_struct_assign
    );

    let (instance_to_struct, main_instance_param) = if let Some(instance) =
        vertex_fn.params.get(1).map(|i| match &i.t {
            ShaderType::Struct(s) => s,
            _ => {
                panic!("Vertex function must take a struct as input");
            }
        }) {
        let instance_to_struct_assign = instance
            .fields
            .iter()
            .map(|f| format!("    instance_input.{} = {};", f.name, f.name))
            .collect::<Vec<String>>()
            .join("\n");

        (
            format!(
                "{} instance_input;\n{}",
                instance.name, instance_to_struct_assign
            ),
            String::from(", instance_input"),
        )
    } else {
        (String::default(), String::default())
    };

    let (out_vars, vertex_out_decl, struct_to_out_assign) = match &vertex_fn.var.t {
        ShaderType::Struct(vertex_out_struct) => {
            let out_vars = vertex_out_struct
                .fields
                .iter()
                .enumerate()
                .map(|(idx, f)| {
                    format!(
                        "layout(location = {}) out {} {}_{};",
                        idx, f.t, vertex_out_struct.name, f.name
                    )
                })
                .collect::<Vec<String>>()
                .join("\n");

            let struct_to_out_assign = vertex_out_struct
                .fields
                .iter()
                .map(|f| {
                    format!(
                        "{}_{} = vertex_output.{};",
                        vertex_out_struct.name, f.name, f.name
                    )
                })
                .collect::<Vec<String>>()
                .join("\n");

            let vertex_out_decl = format!("{} vertex_output =", vertex_fn.var.t);

            (out_vars, vertex_out_decl, struct_to_out_assign)
        }
        ShaderType::Void => (String::default(), String::default(), String::default()),
        _ => {
            panic!(
                "Vertex function must return void or a struct, got {:?}",
                vertex_fn.var.t
            );
        }
    };

    let content = format!(
        r#"
// Structs
{structs}

// Uniforms
{uniforms}

// Buffers
{buffers}

// In
{in_vars}

// Out
{out_vars}

// Functions
{functions}

// Vertex
{vertex_fn}

void main() {{
    // In
    {in_to_struct}

    // Instance
    {instance_to_struct}

    // Out
    {} {}(vertex_input{});
    {}
}}"#,
        vertex_out_decl, vertex_fn.var.name, main_instance_param, struct_to_out_assign
    );

    content
}

pub fn fragment_shader(ProgramInput { content: info, .. }: &ProgramInput) -> String {
    let uniforms = get_uniforms(info);

    let structs = get_structs(info);

    let buffers = get_buffers(info);

    let functions = get_functions(info);

    let frag_fn = if let Some(frag_fn) = info.frag_fn.as_ref() {
        frag_fn
    } else {
        Diagnostic::new(
            Level::Error,
            "No fragment function found when building fragment shader",
        )
        .emit();
        return "".to_string();
    };

    let (in_vars, in_to_struct) = &frag_fn
        .params
        .first()
        .map(|f| {
            let frag_input = &f.t;

            let frag_input = match frag_input {
                ShaderType::Struct(s) => s,
                _ => {
                    panic!(
                        "Fragment function must take a struct as input, got {:?}",
                        frag_input
                    );
                }
            };

            let in_vars = frag_input
                .fields
                .iter()
                .enumerate()
                .map(|(idx, f)| {
                    format!(
                        "layout(location = {}) in {} {}_{};",
                        idx, f.t, frag_input.name, f.name
                    )
                })
                .collect::<Vec<String>>()
                .join("\n");

            let in_to_struct_assign = frag_input
                .fields
                .iter()
                .map(|f| {
                    format!(
                        "    frag_input.{} = {}_{};",
                        f.name, frag_input.name, f.name
                    )
                })
                .collect::<Vec<String>>()
                .join("\n");

            let in_to_struct = format!("{} frag_input;\n{}", frag_input.name, in_to_struct_assign);

            (in_vars, in_to_struct)
        })
        .unwrap_or((String::default(), String::default()));

    let out_var_type = frag_fn.var.t.to_string();

    let fn_param = if in_to_struct.is_empty() {
        String::default()
    } else {
        String::from("frag_input")
    };

    let content = format!(
        r#"
// Structs
{structs}

// Uniforms
{uniforms}

// Buffers
{buffers}

// In
{in_vars}

// Out
layout(location = 0) out {out_var_type} frag_output;

// Functions
{functions}

// Fragment
{frag_fn}

void main() {{
    // In
    {in_to_struct}

    // Out
    frag_output = {}({});
}}"#,
        frag_fn.var.name, fn_param
    );

    content
}

pub fn no_main(ProgramInput { content, .. }: &ProgramInput) -> String {
    let uniforms = get_uniforms(content);

    let structs = get_structs(content);

    let functions = get_functions(content);

    format!(
        r#"// Structs
{structs}

// Uniforms
{uniforms}

//Functions
{functions}
    "#
    )
}
