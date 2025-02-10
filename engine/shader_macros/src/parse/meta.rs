use quote::format_ident;
use syn::Ident;

#[derive(Debug)]
pub struct ProgramMeta {
    pub name: Ident,
    pub version: i32,
}

impl ProgramMeta {
    pub fn ident(&self) -> proc_macro2::Ident {
        format_ident!("Program")
    }

    pub fn vertex_ident(&self) -> proc_macro2::Ident {
        format_ident!("Vertex")
    }

    pub fn uniforms_ident(&self) -> proc_macro2::Ident {
        format_ident!("Uniforms")
    }
}

impl syn::parse::Parse for ProgramMeta {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<syn::Ident>()?;
        input.parse::<syn::Token![,]>()?;

        let mut version = None;

        while input.peek(syn::Ident) {
            let attr = input.parse::<syn::Ident>()?;
            input.parse::<syn::Token![=]>()?;
            match attr.to_string().as_str() {
                "version" | "v" => {
                    let v = input.parse::<syn::LitInt>()?;
                    version = Some(v.base10_parse::<i32>().unwrap());
                }
                _ => {
                    return Err(syn::Error::new(
                        attr.span(),
                        format!("Unknown attribute: {}", attr),
                    ));
                }
            }
        }

        let meta = ProgramMeta {
            name,
            version: version.unwrap_or(460),
        };

        Ok(meta)
    }
}
