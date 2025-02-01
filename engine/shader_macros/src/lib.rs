#![feature(proc_macro_diagnostic, proc_macro_span)]
use proc_macro::{Diagnostic, Level, Span, TokenStream};
use quote::{format_ident, quote};
use shader_var::ShaderVar;

mod parse_glsl;
mod parse_meta;
mod shader_var;

#[derive(Debug)]
struct ShaderMeta {
    version: i32,
}

#[derive(Debug)]
struct ProgramMeta {
    name: String,
    version: i32,
}

impl ProgramMeta {
    pub fn to_vertex_meta(&self) -> ShaderMeta {
        ShaderMeta {
            version: self.version,
        }
    }

    pub fn to_fragment_meta(&self) -> ShaderMeta {
        ShaderMeta {
            version: self.version,
        }
    }

    pub fn to_geometry_meta(&self) -> ShaderMeta {
        ShaderMeta {
            version: self.version,
        }
    }

    pub fn ident(&self) -> proc_macro2::Ident {
        format_ident!("{}", self.name)
    }

    pub fn vertex_ident(&self) -> proc_macro2::Ident {
        format_ident!("{}Vertex", self.name)
    }

    pub fn uniforms_ident(&self) -> proc_macro2::Ident {
        format_ident!("{}Uniforms", self.name)
    }
}

#[derive(Debug)]
enum ShaderError {
    TypeMismatch(Span, ShaderVar, ShaderVar),
    ParseError(Span, String),
    NestedInOutUniform(Span, ShaderVar),
}

#[derive(Debug)]
struct ShaderInput {
    content: String,
    shader_in: Vec<ShaderVar>,
    shader_out: Vec<ShaderVar>,
    shader_uniforms: Vec<ShaderVar>,
    errors: Vec<ShaderError>,
}

impl ShaderInput {
    pub fn check(&self) {
        for error in &self.errors {
            use ShaderError::*;
            match error {
                TypeMismatch(span, a, b) => {
                    Diagnostic::spanned(
                        *span,
                        Level::Error,
                        format!(
                            "{} has type `{:?}` but has type `{:?}` being assigned to it",
                            a.name, a.r#type, b.r#type
                        ),
                    )
                    .emit();
                }
                ParseError(span, msg) => {
                    Diagnostic::spanned(*span, Level::Error, msg).emit();
                }
                NestedInOutUniform(span, var) => {
                    let mut span = span.join(var.name_span).unwrap();
                    if let Some(type_span) = var.type_span {
                        span = span.join(type_span).unwrap();
                    }
                    Diagnostic::spanned(
                        span,
                        Level::Error,
                        "Can't nest an input, output, or uniform",
                    )
                    .emit();
                }
            }
        }
    }
}

#[derive(Debug)]
struct ProgramInput {
    meta: ProgramMeta,
    vertex_shader: ShaderInput,
    fragment_shader: ShaderInput,
    geometry_shader: Option<ShaderInput>,
}

impl ProgramInput {
    pub fn combined_uniforms(&self) -> Vec<ShaderVar> {
        let mut uniforms = self.vertex_shader.shader_uniforms.clone();

        fn insert_uniform(uniforms: &[ShaderVar], list: &mut Vec<ShaderVar>) {
            for uniform in uniforms.iter() {
                let found = list.iter().find(|u| u.name == uniform.name);
                if let Some(found) = found {
                    if found.r#type != uniform.r#type {
                        panic!("Uniforms with the same name must have the same type");
                    }
                } else {
                    list.push(uniform.clone());
                }
            }
        }

        insert_uniform(&self.fragment_shader.shader_uniforms, &mut uniforms);

        if let Some(g) = &self.geometry_shader {
            insert_uniform(&g.shader_uniforms, &mut uniforms);
        }

        uniforms
    }

    pub fn check_in_out(&self) {
        fn compare_in_out(input: &[ShaderVar], output: &[ShaderVar]) {
            for i in input.iter() {
                let found = output.iter().find(|o| o.name == i.name);
                if let Some(found) = found {
                    if found.r#type != i.r#type {
                        let mut span = i.name_span;
                        if let Some(type_span) = i.type_span {
                            span = span.join(type_span).unwrap();
                        }

                        let mut other_span = found.name_span;
                        if let Some(type_span) = found.type_span {
                            other_span = other_span.join(type_span).unwrap();
                        }

                        Diagnostic::spanned(
                            span,
                            Level::Error,
                            format!("Variable {} is an input but with different types", i.name),
                        )
                        .emit();

                        Diagnostic::spanned(
                            other_span,
                            Level::Error,
                            format!("Variable {} is an output but with different types", i.name),
                        )
                        .emit();
                    }
                } else {
                    Diagnostic::spanned(
                        i.name_span,
                        Level::Warning,
                        format!("Output variable {} is not present in input", i.name),
                    )
                    .emit();
                }
            }
        }

        let vertex_send = &self.vertex_shader.shader_out;
        let vertex_receive = if let Some(g) = &self.geometry_shader {
            &g.shader_in
        } else {
            &self.fragment_shader.shader_in
        };

        compare_in_out(vertex_send, vertex_receive);

        if let Some(g) = self.geometry_shader.as_ref() {
            let geometry_send = &g.shader_out;
            let geometry_receive = &self.fragment_shader.shader_in;

            compare_in_out(geometry_send, geometry_receive);
        }
    }

    pub fn check(&self) {
        self.check_in_out();

        self.vertex_shader.check();
        self.fragment_shader.check();
        if let Some(g) = &self.geometry_shader {
            g.check();
        }
    }
}

fn make_vertex_shader(
    ident: &proc_macro2::Ident,
    shader: &ShaderInput,
) -> proc_macro2::TokenStream {
    let shader_in = &shader.shader_in;

    let shader_in_names = shader_in.iter().map(|s| format_ident!("{}", s.name));
    let vertex_impl = quote! {
        ::glium::implement_vertex!(#ident #(, #shader_in_names)*);
    };

    quote! {
        #[derive(Debug, Copy, Clone)]
        pub struct #ident {
            #(pub #shader_in)*
        }

        #vertex_impl
    }
}

fn make_uniforms(ident: &proc_macro2::Ident, uniforms: &[ShaderVar]) -> proc_macro2::TokenStream {
    quote! {
    pub struct #ident {
        #(pub #uniforms)*
    }

        }
}

fn make_program(ident: &proc_macro2::Ident, program: &ProgramInput) -> proc_macro2::TokenStream {
    let vert_content = &program.vertex_shader.content;
    let frag_content = &program.fragment_shader.content;
    let geom_content = &program.geometry_shader.as_ref().map(|g| &g.content);

    let vert_source = proc_macro2::Literal::string(vert_content.as_str());
    let frag_source = proc_macro2::Literal::string(frag_content.as_str());
    let geom_source = if let Some(g) = geom_content {
        let g = proc_macro2::Literal::string(g.as_str());
        quote! { Some(#g) }
    } else {
        quote! { None }
    };

    quote! {
        pub struct #ident;

        impl ::shaders::ProgramInternal for #ident {
            fn vertex() -> &'static str {
                #vert_source
            }

            fn fragment() -> &'static str {
                #frag_source
            }

            fn geometry() -> Option<&'static str> {
                #geom_source
            }
        }
    }
}

#[proc_macro]
pub fn program(input: TokenStream) -> TokenStream {
    let mut iter = input.into_iter();

    let (meta, vertex_content, fragment_content, geometry_content) =
        parse_meta::parse_program_meta(&mut iter);

    let vertex_shader = parse_glsl::parse_glsl(vertex_content, meta.to_vertex_meta());
    let fragment_shader = parse_glsl::parse_glsl(fragment_content, meta.to_fragment_meta());
    let geometry_shader =
        geometry_content.map(|g| parse_glsl::parse_glsl(g, meta.to_geometry_meta()));

    let program = ProgramInput {
        meta,
        vertex_shader,
        fragment_shader,
        geometry_shader,
    };

    // combine all uniforms from all shaders
    let uniforms = program.combined_uniforms();

    let vertex = make_vertex_shader(&program.meta.vertex_ident(), &program.vertex_shader);
    let uniforms = make_uniforms(&program.meta.uniforms_ident(), &uniforms);

    program.check();

    let program = make_program(&program.meta.ident(), &program);

    let expanded = quote! {
        #vertex

        #uniforms

        #program
    };

    expanded.into()
}
