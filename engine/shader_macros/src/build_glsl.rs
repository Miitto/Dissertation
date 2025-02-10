use crate::{
    ProgramInput, ShaderInfo,
    shader_var::{ShaderObjects, ShaderType},
};

fn get_uniforms(info: &ShaderInfo) -> String {
    info.uniforms
        .iter()
        .map(|uniform| format!("uniform {} {};", uniform.t, uniform.name))
        .collect::<Vec<String>>()
        .join("\n")
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

pub fn vertex_shader(
    ProgramInput {
        content: info,
        meta,
    }: &ProgramInput,
) -> String {
    let uniforms = get_uniforms(info);

    let structs = get_structs(info);

    let functions = get_functions(info);

    let vertex_fn = info.vertex_fn.as_ref().expect("No vertex function found");

    let vertex_input = match &vertex_fn.params[0].t {
        ShaderType::Object(ShaderObjects::Custom(s)) => s,
        _ => {
            panic!("Fragment function must take a struct as input");
        }
    };

    let in_vars = {
        let iter = vertex_input.fields.iter();

        if let Some(instance) = vertex_fn.params.get(1).map(|i| match &i.t {
            ShaderType::Object(ShaderObjects::Custom(s)) => s,
            _ => {
                panic!("Vertex function must take a struct as input");
            }
        }) {
            let iter = iter.chain(instance.fields.iter());
            iter.map(|f| format!("in {} {};", f.t, f.name))
                .collect::<Vec<String>>()
                .join("\n")
        } else {
            iter.map(|f| format!("in {} {};", f.t, f.name))
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
            ShaderType::Object(ShaderObjects::Custom(s)) => s,
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

    let vertex_out_struct = match &vertex_fn.return_type {
        ShaderType::Object(ShaderObjects::Custom(s)) => s,
        _ => {
            panic!("Fragment function must take a struct as input");
        }
    };

    let out_vars = vertex_out_struct
        .fields
        .iter()
        .map(|f| format!("out {} {}_{};", f.t, vertex_out_struct.name, f.name))
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

    {instance_to_struct}

    // Out
    {} vertex_output = {}(vertex_input{});
    {}
}}"#,
        meta.version,
        vertex_fn.return_type,
        vertex_fn.name,
        main_instance_param,
        struct_to_out_assign
    );

    content
}

pub fn fragment_shader(
    ProgramInput {
        content: info,
        meta,
    }: &ProgramInput,
) -> String {
    let uniforms = get_uniforms(info);

    let structs = get_structs(info);

    let functions = get_functions(info);

    let frag_fn = info.frag_fn.as_ref().expect("No vertex function found");

    let frag_input = match &frag_fn.params[0].t {
        ShaderType::Object(ShaderObjects::Custom(s)) => s,
        _ => {
            panic!("Fragment function must take a struct as input");
        }
    };

    let in_vars = frag_input
        .fields
        .iter()
        .map(|f| format!("in {} {}_{};", f.t, frag_input.name, f.name))
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
    frag_output = {}(frag_input);
}}"#,
        meta.version, frag_fn.name
    );

    content
}

#[expect(unreachable_code, unused_variables)]
pub fn geometry_shader(
    ProgramInput {
        content: info,
        meta,
    }: &ProgramInput,
) -> Option<String> {
    return None;
    let uniforms = get_uniforms(info);

    let structs = get_structs(info);

    let functions = get_functions(info);

    info.geometry_fn.as_ref().map(|geom_fn| {
        let geom_input = match &geom_fn.params[0].t {
            ShaderType::Object(ShaderObjects::Custom(s)) => s,
            _ => {
                panic!("Fragment function must take a struct as input");
            }
        };

        let in_vars = geom_input
            .fields
            .iter()
            .map(|f| format!("in {} {};", f.t, f.name))
            .collect::<Vec<String>>()
            .join("\n");

        let in_to_struct_assign = geom_input
            .fields
            .iter()
            .map(|f| format!("    geom_input.{} = {};", f.name, f.name))
            .collect::<Vec<String>>()
            .join("\n");

        let in_to_struct = format!("{} geom_input;\n{}", geom_input.name, in_to_struct_assign);

        let geom_out_struct = match &geom_fn.return_type {
            ShaderType::Object(ShaderObjects::Custom(s)) => s,
            _ => {
                panic!("Fragment function must take a struct as input");
            }
        };

        let out_vars = geom_out_struct
            .fields
            .iter()
            .map(|f| format!("out {} {}_{};", f.t, geom_out_struct.name, f.name))
            .collect::<Vec<String>>()
            .join("\n");

        let struct_to_out_assign = geom_out_struct
            .fields
            .iter()
            .map(|f| {
                format!(
                    "{}_{} = geom_output.{};",
                    geom_out_struct.name, f.name, f.name
                )
            })
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
    {} geom_output = {}(input);
    {}
}}"#,
            meta.version, geom_fn.return_type, geom_fn.name, struct_to_out_assign
        );

        content
    })
}
