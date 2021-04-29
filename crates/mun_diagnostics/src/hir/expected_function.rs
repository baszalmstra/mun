use super::HirDiagnostic;
use crate::{Diagnostic, SourceAnnotation};
use mun_hir::HirDisplay;
use mun_syntax::TextRange;

/// An error that is emitted when a function is expected but something else is encountered:
///
/// ```mun
/// # fn main() {
///     let a = 3;
///     let b = a();    // expected function
/// # }
/// ```
pub struct ExpectedFunction<'db, 'diag> {
    db: &'db dyn mun_hir::HirDatabase,
    diag: &'diag mun_hir::diagnostics::ExpectedFunction,
}

impl<'db, 'diag> Diagnostic for ExpectedFunction<'db, 'diag> {
    fn range(&self) -> TextRange {
        self.diag.highlight_range()
    }

    fn title(&self) -> String {
        format!(
            "expected function, found `{}`",
            self.diag.found.display(self.db)
        )
    }

    fn primary_annotation(&self) -> Option<SourceAnnotation> {
        Some(SourceAnnotation {
            range: self.diag.highlight_range(),
            message: "not a function".to_owned(),
        })
    }
}

impl<'db, 'diag> ExpectedFunction<'db, 'diag> {
    /// Constructs a new instance of `ExpectedFunction`
    pub fn new(
        db: &'db dyn mun_hir::HirDatabase,
        diag: &'diag mun_hir::diagnostics::ExpectedFunction,
    ) -> Self {
        ExpectedFunction { db, diag }
    }
}
