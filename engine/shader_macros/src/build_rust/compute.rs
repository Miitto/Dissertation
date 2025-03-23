use quote::{format_ident, quote};
pub fn compute_program(
    source: &str,
    name: &str,
    uses: &[Vec<proc_macro::Ident>],
    version: u32,
    use_crate: bool,
) -> proc_macro2::TokenStream {
    let crate_path = if use_crate {
        quote! {crate}
    } else {
        quote! {renderer}
    };

    let uses = uses
        .iter()
        .map(|s| {
            let idents = s.iter().map(|i| format_ident!("{}", i.to_string()));
            quote! {#(#idents)::*::SOURCE}
        })
        .reduce(|acc, el| quote! {combine!(#acc, #el)})
        .unwrap_or(quote! {"\n"});

    let name = format_ident!("{}", name);
    let version = format!("#version {}\n", version);

    quote! {
        pub struct #name;

        impl #crate_path::ComputeProgramInternal for #name {
            fn compute() -> &'static str {
                macro_rules! combine {
                    ($A:expr, $B:expr) => {{
                        const A: &str = $A;
                        const B: &str = $B;
                        const LEN: usize = A.len() + B.len();
                        const fn combined() -> [u8; LEN] {
                            let mut out = [0u8; LEN];
                            out = copy_slice(A.as_bytes(), out, 0);
                            out = copy_slice(B.as_bytes(), out, A.len());
                            out
                        }
                        const fn copy_slice(input: &[u8], mut output: [u8; LEN], offset: usize) -> [u8; LEN] {
                            let mut index = 0;
                            loop {
                                output[offset + index] = input[index];
                                index += 1;
                                if index == input.len() {
                                    break;
                                }
                            }
                            output
                        }
                        const RESULT: &[u8] = &combined();
                        // how bad is the assumption that `&str` and `&[u8]` have the same layout?
                        const RESULT_STR: &str = unsafe { std::str::from_utf8_unchecked(RESULT) };
                        RESULT_STR
                    }};
                }

                const USES_SOURCE: &'static str = #uses;

                const VERSIONED: &'static str = combine!(#version, USES_SOURCE);

                const NEW_LINED: &'static str = combine!(VERSIONED, "\n");

                const COMPUTE: &'static str = combine!(NEW_LINED, #source);
                COMPUTE

            }
        }
    }
}
