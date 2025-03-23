use crate::ShaderInfo;
use quote::{format_ident, quote};

mod buffers;
mod compute;
mod program;
mod uniforms;
mod vertex;

pub use buffers::*;
pub use compute::*;
pub use program::*;
pub use uniforms::*;
pub use vertex::*;

pub fn struct_defs(info: &ShaderInfo) -> proc_macro2::TokenStream {
    let structs = &info.structs;

    quote! {
        pub mod structs {
            #(#[derive(Debug)]pub #structs)*
        }
    }
}

pub fn uses(info: &ShaderInfo) -> proc_macro2::TokenStream {
    let uses = info
        .uses
        .iter()
        .map(|s| {
            let idents = s.iter().map(|i| format_ident!("{}", i.to_string()));

            quote! { pub use #(#idents)::*;}
        })
        .collect::<Vec<_>>();

    quote! {
        pub mod uses {
            #(#uses)*
        }
    }
}
