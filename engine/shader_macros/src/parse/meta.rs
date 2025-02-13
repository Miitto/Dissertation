use proc_macro::{Diagnostic, Ident, Level, TokenTree};

use crate::parse::{punct, uint};

use super::ident_any;

use crate::Result;

#[derive(Debug)]
pub struct ProgramMeta {
    pub name: Ident,
    pub version: u32,
}

#[expect(clippy::while_let_loop)]
pub fn parse_meta(input: &[TokenTree]) -> Result<&[TokenTree], ProgramMeta, Diagnostic> {
    let (input, name) = ident_any(input)
        .map_err(|_| Diagnostic::spanned(input[0].span(), Level::Error, "Expected Shader name"))?;
    let (mut input, _) = punct(',')(input)
        .map_err(|_| Diagnostic::spanned(input[0].span(), Level::Error, "Expected ,"))?;

    let mut version = 460;

    fn parse_version(input: &[TokenTree]) -> Result<&[TokenTree], u32> {
        let (input, _) = ident_any(input)?;
        let (input, _) = punct('=')(input)?;
        let (input, v) = uint(input)?;

        Ok((input, v))
    }

    loop {
        if let Ok((rest, v)) = parse_version(input) {
            input = rest;
            version = v;
        } else {
            break;
        }
    }

    Ok((
        input,
        ProgramMeta {
            name: name.clone(),
            version,
        },
    ))
}
