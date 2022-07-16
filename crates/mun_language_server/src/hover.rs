//! This modules contains everything that has to do with computing hover information at a given
//! position in the source.

use crate::{analysis::RangeInfo, db::AnalysisDatabase, markup::Markup, FilePosition};
use hir::semantics::Semantics;
use mun_syntax::{AstNode, SyntaxKind, SyntaxKind::*, SyntaxToken, TokenAtOffset, T};

/// Contains the results when hovering over a token
#[derive(Debug, Default)]
pub struct HoverResult {
    pub markup: Markup,
}

/// This is the main entry point for computing hover information. It is used to show additional
/// information, like the type of an expression or the documentation for a definition.
///
/// This functionality is usually triggered by hovering over an item in code but it can often also
/// be triggered with a shortcut.
pub(crate) fn hover(
    db: &AnalysisDatabase,
    position: FilePosition,
) -> Option<RangeInfo<HoverResult>> {
    let semantics = Semantics::new(db);
    let file = semantics.parse(position.file_id).syntax().clone();

    // Pick the token that is most likely what the user is hovering.
    let original_token =
        pick_best_token(file.token_at_offset(position.offset), |kind| match kind {
            IDENT | INT_NUMBER => 3,
            T!['('] | T![')'] => 2,
            kind if kind.is_trivia() => 0,
            _ => 1,
        })?;

    None
}

/// Picks the token with the highest rank returned by the passed in function.
pub fn pick_best_token(
    tokens: TokenAtOffset<SyntaxToken>,
    f: impl Fn(SyntaxKind) -> usize,
) -> Option<SyntaxToken> {
    tokens.max_by_key(move |t| f(t.kind()))
}
