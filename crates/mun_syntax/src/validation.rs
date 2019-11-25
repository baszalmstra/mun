use crate::{ast, AstNode, SyntaxError, SyntaxErrorKind, SyntaxNode};

/// Iterate over all nodes and its descendants performing extra syntax validation steps that are
/// harder to detect when parsing.
pub(crate) fn validate(root: &SyntaxNode) -> Vec<SyntaxError> {
    let mut errors = Vec::new();
    for node in root.descendants() {
        match_ast! {
            match node {
                ast::RangeExpr(it) => { validate_range_expr(it, &mut errors) },
                _ => {},
            }
        }
    }
    errors
}

fn validate_range_expr(expr: ast::RangeExpr, errors: &mut Vec<SyntaxError>) {
    if expr.op_kind() == Some(ast::RangeOp::Inclusive) && expr.end().is_none() {
        errors.push(SyntaxError::new(
            SyntaxErrorKind::InclusiveRangeMissingEnd,
            expr.syntax().text_range(),
        ))
    }
}
