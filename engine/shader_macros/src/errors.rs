use proc_macro::{Diagnostic, Level, Span};

use crate::shader_var::ShaderVar;

#[derive(Debug)]
#[expect(dead_code)]
pub(crate) enum ShaderError {
    TypeMismatch(Span, ShaderVar, ShaderVar),
    ParseError(Span, String),
    UnknownVariable(Span, String),
    UnknownType(Span, String),
}

pub(crate) fn diagnostics(errors: &[ShaderError]) {
    for error in errors {
        use ShaderError::*;
        match error {
            ParseError(span, msg) => {
                Diagnostic::spanned(*span, Level::Error, msg.clone()).emit();
            }
            TypeMismatch(span, a, b) => {
                Diagnostic::spanned(
                    *span,
                    Level::Error,
                    format!("Type mismatch: {} is being assigned {}", a, b),
                )
                .emit();
            }
            UnknownVariable(span, name) => {
                Diagnostic::spanned(*span, Level::Error, format!("Unknown variable: {}", name))
                    .emit();
            }
            UnknownType(span, name) => {
                Diagnostic::spanned(*span, Level::Error, format!("Unknown type: {}", name)).emit();
            }
        }
    }
}
