use super::HirDiagnostic;
use crate::{Diagnostic, SourceAnnotation};
use mun_hir::HirDisplay;
use mun_syntax::{ast, AstNode, TextRange};

/// An error that is emitted when trying to access a field that doesn't exist.
///
/// ```mun
/// struct Foo {
///     b: i32
/// }
///
/// # fn main() {
/// let a = Foo { b: 3}
/// let b = a.c;    // no field `c`
/// #}
/// ```
pub struct AccessUnknownField<'db, 'diag> {
    db: &'db dyn mun_hir::HirDatabase,
    diag: &'diag mun_hir::diagnostics::AccessUnknownField,
    location: TextRange,
}

impl<'db, 'diag> Diagnostic for AccessUnknownField<'db, 'diag> {
    fn range(&self) -> TextRange {
        self.location
    }

    fn title(&self) -> String {
        format!(
            "no field `{}` on type `{}`",
            self.diag.name,
            self.diag.receiver_ty.display(self.db),
        )
    }

    fn primary_annotation(&self) -> Option<SourceAnnotation> {
        Some(SourceAnnotation {
            range: self.location,
            message: "unknown field".to_string(),
        })
    }
}

impl<'db, 'diag> AccessUnknownField<'db, 'diag> {
    /// Constructs a new instance of `AccessUnknownField`
    pub fn new(
        db: &'db dyn mun_hir::HirDatabase,
        diag: &'diag mun_hir::diagnostics::AccessUnknownField,
    ) -> Self {
        let parse = db.parse(diag.file);

        let location = ast::FieldExpr::cast(diag.expr.to_node(&parse.syntax_node()))
            .map(|f| f.field_range())
            .unwrap_or_else(|| diag.highlight_range());

        AccessUnknownField { db, diag, location }
    }
}
