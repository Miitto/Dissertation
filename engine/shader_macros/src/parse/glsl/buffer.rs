use proc_macro::{Diagnostic, Level, TokenTree};

use crate::{
    ShaderInfo,
    parse::{delimited, ident, ident_any, punct},
    uniform::LayoutBlock,
};

use super::{State, r#struct::parse_layout_block};

use crate::Result;

pub fn parse_buffer<'a>(
    input: &'a [TokenTree],
    info: &mut ShaderInfo,
    state: &mut State,
) -> Result<&'a [TokenTree], LayoutBlock, Option<Diagnostic>> {
    let (input, _) = ident("buffer")(input).map_err(|_| None)?;

    let (input, st) = parse_layout_block(input, info)?;

    let (input, var_name) = ident_any(input)
        .map(|(i, v)| (i, Some(v.clone())))
        .unwrap_or((input, None));

    let (input, is_array) = if let Ok((i, _)) = delimited(proc_macro::Delimiter::Bracket)(input) {
        (i, true)
    } else {
        (input, false)
    };

    let (input, _) = punct(';')(input).map_err(|_| {
        if input.is_empty() {
            Diagnostic::new(Level::Error, "Unexpected end of input, expected ;")
        } else {
            Diagnostic::spanned(input[0].span(), Level::Error, "Expected ;")
        }
    })?;

    let bind = if let Some(bind) = state.next_bind.take() {
        bind
    } else {
        return Err(Some(Diagnostic::spanned(
            st.name.span(),
            Level::Error,
            "Expected #bind directive before this line",
        )));
    };

    let buffer = LayoutBlock {
        bind,
        name: st.name,
        fields: st.fields,
        var_name,
        is_array,
    };

    Ok((input, buffer))
}
