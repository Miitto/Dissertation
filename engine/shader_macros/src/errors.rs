use proc_macro::{Diagnostic, Level, Span};

use crate::shader_var::ShaderVar;

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum ShaderError {
    TypeMismatch {
        assign_loc: Span,
        assignee: ShaderVar,
        assigner: ShaderVar,
    },
    ParseError(Span, String),
    UnknownVariable(Span, String),
    UnknownType(Span, String),
}

impl ShaderError {
    pub(crate) fn emit(&self) {
        use ShaderError::*;
        match self {
            ParseError(span, msg) => {
                Diagnostic::spanned(*span, Level::Error, msg.clone()).emit();
            }
            TypeMismatch {
                assign_loc,
                assignee,
                assigner,
            } => {
                Diagnostic::spanned(
                    *assign_loc,
                    Level::Error,
                    format!("Type mismatch: {} is being assigned {}", assignee, assigner),
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
