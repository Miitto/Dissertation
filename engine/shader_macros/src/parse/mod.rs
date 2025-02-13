use proc_macro::{Delimiter, Ident, Punct, TokenTree};

pub mod glsl;
pub mod meta;

use crate::Result;

pub fn ident_any(input: &[TokenTree]) -> Result<&[TokenTree], &Ident>
where
{
    if let Some(TokenTree::Ident(ident)) = input.first() {
        Ok((&input[1..], ident))
    } else {
        Err(())
    }
}

pub fn ident<'a>(tag: &str) -> impl Fn(&'a [TokenTree]) -> Result<&'a [TokenTree], &'a Ident>
where
{
    move |i: &'a [TokenTree]| {
        if let Some(TokenTree::Ident(ident)) = i.first() {
            if ident.to_string() == tag {
                return Ok((&i[1..], ident));
            }
        }

        Err(())
    }
}

pub fn punct<'a>(punct: char) -> impl Fn(&'a [TokenTree]) -> Result<&'a [TokenTree], &'a Punct>
where
{
    move |i: &'a [TokenTree]| {
        if let Some(TokenTree::Punct(p)) = i.first() {
            if p.as_char() == punct {
                return Ok((&i[1..], p));
            }
        }

        Err(())
    }
}

pub fn string(input: &[TokenTree]) -> Result<&[TokenTree], String> {
    if let Some(TokenTree::Literal(lit)) = input.first() {
        let s = lit.to_string();

        if !s.starts_with('"') || !s.ends_with('"') {
            return Err(());
        }

        return Ok((&input[1..], s[1..s.len() - 1].to_string()));
    }

    Err(())
}

pub fn uint(input: &[TokenTree]) -> Result<&[TokenTree], u32> {
    if let Some(TokenTree::Literal(lit)) = input.first() {
        if let Ok(lit) = lit.to_string().parse() {
            return Ok((&input[1..], lit));
        }
    }

    Err(())
}

#[allow(dead_code)]
pub struct Delimited {
    pub delimiter: Delimiter,
    pub content: Vec<TokenTree>,
    pub span: proc_macro::Span,
}

#[allow(dead_code)]
pub fn delimited_any(input: &[TokenTree]) -> Result<&[TokenTree], Delimited> {
    if let Some(TokenTree::Group(group)) = input.first() {
        let stream = group.stream();
        let collected = stream.into_iter().collect::<Vec<_>>();
        Ok((
            &input[1..],
            Delimited {
                delimiter: group.delimiter(),
                content: collected,
                span: group.span(),
            },
        ))
    } else {
        Err(())
    }
}

pub fn delimited<'a>(
    delimiter: Delimiter,
) -> impl Fn(&'a [TokenTree]) -> Result<&'a [TokenTree], Delimited> {
    move |i: &'a [TokenTree]| {
        if let Some(TokenTree::Group(g)) = i.first() {
            if g.delimiter() == delimiter {
                let stream = g.stream();
                let collected = stream.into_iter().collect::<Vec<_>>();
                return Ok((
                    &i[1..],
                    Delimited {
                        delimiter: g.delimiter(),
                        content: collected,
                        span: g.span(),
                    },
                ));
            }
        }

        Err(())
    }
}
