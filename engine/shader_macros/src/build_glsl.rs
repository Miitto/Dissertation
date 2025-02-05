use crate::{ProgramMeta, link_info::LinkedShaderInfo};

fn get_uniforms(info: &LinkedShaderInfo) -> String {
    info.uniforms
        .iter()
        .map(|uniform| format!("uniform {} {};", uniform.t, uniform.name))
        .collect::<Vec<String>>()
        .join("\n")
}

fn get_structs(info: &LinkedShaderInfo) -> String {
    info.structs
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
        .join("\n\n")
}

fn get_functions(info: &LinkedShaderInfo) -> String {
    info.functions
        .iter()
        .map(|f| f.to_string())
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn build_vertex_shader(info: &LinkedShaderInfo, meta: &ProgramMeta) -> String {
    let uniforms = get_uniforms(info);

    let structs = get_structs(info);

    let functions = get_functions(info);

    let vertex_fn = info.vertex_fn.as_ref().expect("No vertex function found");

    let vertex_input = vertex_fn.params[0].t.get_struct();

    let in_vars = vertex_input
        .fields
        .iter()
        .map(|f| format!("in {} {};", f.r#type.to_glsl(), f.name))
        .collect::<Vec<String>>()
        .join("\n");

    let in_to_struct_assign = vertex_input
        .fields
        .iter()
        .map(|f| format!("    input.{} = {};", f.name, f.name))
        .collect::<Vec<String>>()
        .join("\n");

    let in_to_struct = format!("{} input;\n{}", vertex_input.name, in_to_struct_assign);

    let vertex_out_struct = vertex_fn.return_type.get_struct();

    let out_vars = vertex_out_struct
        .fields
        .iter()
        .map(|f| {
            format!(
                "out {} {}_{};",
                f.r#type.to_glsl(),
                vertex_out_struct.name,
                f.name
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    let struct_to_out_assign = vertex_out_struct
        .fields
        .iter()
        .map(|f| format!("{}_{} = output.{};", vertex_out_struct.name, f.name, f.name))
        .collect::<Vec<String>>()
        .join("\n");

    let content = format!(
        r#"#version {}

// Structs
{structs}

// Uniforms
{uniforms}

// Functions
{functions}

// Vertex
{vertex_fn}

// In
{in_vars}

// Out
{out_vars}

void main() {{
    // In
    {in_to_struct}

    // Out
    {} output = {}(input);
    {}
}}"#,
        meta.version, vertex_fn.return_type, vertex_fn.name, struct_to_out_assign
    );

    content
}

pub fn build_fragment_shader(info: &LinkedShaderInfo, meta: &ProgramMeta) -> String {
    let uniforms = get_uniforms(info);

    let structs = get_structs(info);

    let functions = get_functions(info);

    let frag_fn = info.frag_fn.as_ref().expect("No vertex function found");

    let frag_input = frag_fn.params[0].t.get_struct();

    let in_vars = frag_input
        .fields
        .iter()
        .map(|f| format!("in {} {}_{};", f.r#type.to_glsl(), frag_input.name, f.name))
        .collect::<Vec<String>>()
        .join("\n");

    let in_to_struct_assign = frag_input
        .fields
        .iter()
        .map(|f| format!("    input.{} = {}_{};", f.name, frag_input.name, f.name))
        .collect::<Vec<String>>()
        .join("\n");

    let in_to_struct = format!("{} input;\n{}", frag_input.name, in_to_struct_assign);

    let out_var_type = frag_fn.return_type.to_string();

    let content = format!(
        r#"#version {}

// Structs
{structs}

// Uniforms
{uniforms}

// Functions
{functions}

// Fragment
{frag_fn}

// In
{in_vars}

// Out
out {out_var_type} frag_output;

void main() {{
    // In
    {in_to_struct}

    // Out
    frag_output = {}(input);
}}"#,
        meta.version, frag_fn.name
    );

    content
}

#[expect(unreachable_code, unused_variables)]
pub fn build_geometry_shader(info: &LinkedShaderInfo, meta: &ProgramMeta) -> Option<String> {
    return None;
    let uniforms = get_uniforms(info);

    let structs = get_structs(info);

    let functions = get_functions(info);

    info.geometry_fn.as_ref().map(|geom_fn| {
        let geom_input = geom_fn.params[0].t.get_struct();

        let in_vars = geom_input
            .fields
            .iter()
            .map(|f| format!("in {} {};", f.r#type.to_glsl(), f.name))
            .collect::<Vec<String>>()
            .join("\n");

        let in_to_struct_assign = geom_input
            .fields
            .iter()
            .map(|f| format!("    input.{} = {};", f.name, f.name))
            .collect::<Vec<String>>()
            .join("\n");

        let in_to_struct = format!("{} input;\n{}", geom_input.name, in_to_struct_assign);

        let vertex_out_struct = geom_fn.return_type.get_struct();

        let out_vars = vertex_out_struct
            .fields
            .iter()
            .map(|f| {
                format!(
                    "out {} {}_{};",
                    f.r#type.to_glsl(),
                    vertex_out_struct.name,
                    f.name
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        let struct_to_out_assign = vertex_out_struct
            .fields
            .iter()
            .map(|f| format!("{}_{} = output.{};", vertex_out_struct.name, f.name, f.name))
            .collect::<Vec<String>>()
            .join("\n");

        let content = format!(
            r#"#version {}

// Structs
{structs}

// Uniforms
{uniforms}

// Functions
{functions}

// Geometry
{geom_fn}

// In
{in_vars}

// Out
{out_vars}

void main() {{
    // In
    {in_to_struct}

    // Out
    {} output = {}(input);
    {}
}}"#,
            meta.version, geom_fn.return_type, geom_fn.name, struct_to_out_assign
        );

        content
    })
}
