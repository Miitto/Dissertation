use proc_macro::{Span, TokenTree};

use crate::{
    ShaderInfo,
    errors::ShaderError,
    shader_var::{ShaderType, ShaderVar},
};

fn check_assignment(
    var: &ShaderVar,
    line: &[TokenTree],
    local_vars: &[ShaderVar],
    info: &ShaderInfo,
) {
    if !line.iter().any(|t| t.is_punct('=')) {
        return;
    }

    let (member_end, var_type) = member_walk(&var.t, &line[1..]);
    let member_end = member_end + 1;

    if let Some(var_type) = var_type {
        let member_line = &line[..member_end];
        let rest_line = &line[member_end + 1..];

        if member_line.is_empty() || rest_line.is_empty() {
            return;
        }

        let assign_type = walk(rest_line, local_vars, info);
        if let Some(assign_type) = assign_type {
            if var_type != assign_type {
                let rest_first_span = rest_line.first().unwrap().span();
                let rest_last_span = rest_line.last().unwrap().span();
                let rest_span = rest_first_span
                    .join(rest_last_span)
                    .unwrap_or(rest_first_span);

                let member_first_span = member_line.first().unwrap().span();
                let member_last_span = member_line.last().unwrap().span();
                let member_span = member_first_span
                    .join(member_last_span)
                    .unwrap_or(member_first_span);

                let member_line_str = member_line
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<String>();
                let rest_line_str = rest_line.iter().map(|t| t.to_string()).collect::<String>();

                ShaderError::TypeMismatch {
                    assign_loc: rest_span,
                    assignee: ShaderVar {
                        name: member_line_str,
                        name_span: member_span,
                        t: var_type,
                        type_span: None,
                    },
                    assigner: ShaderVar {
                        name: rest_line_str,
                        name_span: rest_span,
                        t: assign_type,
                        type_span: None,
                    },
                }
                .emit();
            }
        }
    }
}

fn check_initialization(
    var_type: ShaderType,
    type_span: Span,
    line: &[TokenTree],
    local_vars: &mut Vec<ShaderVar>,
    info: &ShaderInfo,
) {
    let new_var = ShaderVar::new(
        var_type,
        Some(type_span),
        line[0].to_string(),
        line[0].span(),
    );

    local_vars.push(new_var);

    let new_var = local_vars
        .last()
        .expect("Failed to get item we just pushed");

    check_assignment(new_var, line, local_vars, info);
}

pub(crate) fn type_check(line: &[TokenTree], local_vars: &mut Vec<ShaderVar>, info: &ShaderInfo) {
    if line.len() < 2 {
        return;
    }

    let first = &line[0];
    if let TokenTree::Ident(ident) = first {
        if let Some(var) = info.get_var(ident, local_vars) {
            check_assignment(&var, line, local_vars, info);
            return;
        }
    }

    // Don't pass local vars as we don't want to accidentally capture a variables type instad of the type itself
    let var_type = info.get_type(first, &[]);

    if !matches!(var_type, ShaderType::Unknown(_)) {
        check_initialization(var_type, first.span(), &line[1..], local_vars, info);
    }
}

fn member_walk(in_type: &ShaderType, line: &[TokenTree]) -> (usize, Option<ShaderType>) {
    if line.len() < 2 || !line[0].is_punct('.') {
        return (0, Some(in_type.clone()));
    }

    let member = &line[1];
    if let TokenTree::Ident(ident) = member {
        if let Some(member_type) = in_type.get_member(ident) {
            let (count, member_type) = member_walk(&member_type, &line[2..]);
            return (count + 2, member_type);
        }
    }

    (0, None)
}

fn walk(line: &[TokenTree], local_vars: &[ShaderVar], info: &ShaderInfo) -> Option<ShaderType> {
    if line.is_empty() {
        return None;
    }

    if !line.is_empty() {
        let var_type = info.get_type(&line[0], local_vars);

        if !matches!(var_type, ShaderType::Unknown(_)) {
            let (member_len, var_type) = member_walk(&var_type, &line[1..]);
            if let Some(var_type) = var_type {
                let rest_line = &line[member_len + 1..];

                if rest_line.is_empty() {
                    return Some(var_type);
                }

                if let TokenTree::Punct(p) = &rest_line[0] {
                    let right_var = info.get_type(&rest_line[1], local_vars);
                    if !matches!(right_var, ShaderType::Unknown(_)) {
                        let (_moved, right) = member_walk(&right_var, &rest_line[2..]);
                        if let Some(right) = right {
                            let left = var_type;

                            let res = match p.as_char() {
                                '*' => left * right,
                                '/' => left / right,
                                '+' => left + right,
                                '-' => left - right,
                                _ => {
                                    return None;
                                }
                            };

                            if let Err(err) = &res {
                                ShaderError::ParseError(rest_line[0].span(), err.clone()).emit();
                            }

                            return res.ok();
                        }
                    }
                }
            }
        }
        return Some(info.get_type(&line[0], local_vars));
    }

    // TODO: Check operations
    None
}

trait TokenIsPunct {
    fn is_punct(&self, c: char) -> bool;
}

impl TokenIsPunct for TokenTree {
    fn is_punct(&self, c: char) -> bool {
        if let TokenTree::Punct(p) = self {
            p.as_char() == c
        } else {
            false
        }
    }
}
