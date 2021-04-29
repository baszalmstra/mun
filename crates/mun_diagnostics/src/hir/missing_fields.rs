use super::HirDiagnostic;
use crate::{Diagnostic, SourceAnnotation};
use mun_hir::HirDisplay;
use mun_syntax::{ast, AstNode, TextRange};

/// An error that is emitted when a field is missing from a struct initializer.
///
/// ```mun
/// struct Foo {
///     a: i32,
/// }
///
/// # fn main() {
///     let a = Foo {}; // missing field `a`
/// # }
/// ```
pub struct MissingFields<'db, 'diag> {
    db: &'db dyn mun_hir::HirDatabase,
    diag: &'diag mun_hir::diagnostics::MissingFields,
    location: TextRange,
    missing_fields: String,
}

impl<'db, 'diag> Diagnostic for MissingFields<'db, 'diag> {
    fn range(&self) -> TextRange {
        self.location
    }

    fn title(&self) -> String {
        format!(
            "missing fields {} in initializer of `{}`",
            self.missing_fields,
            self.diag.struct_ty.display(self.db)
        )
    }

    fn primary_annotation(&self) -> Option<SourceAnnotation> {
        Some(SourceAnnotation {
            range: self.location,
            message: format!("missing {}", self.missing_fields.clone()),
        })
    }
}

impl<'db, 'diag> MissingFields<'db, 'diag> {
    /// Constructs a new instance of `MissingFields`
    pub fn new(
        db: &'db dyn mun_hir::HirDatabase,
        diag: &'diag mun_hir::diagnostics::MissingFields,
    ) -> Self {
        let parse = db.parse(diag.file);
        let missing_fields = diag
            .field_names
            .iter()
            .map(|n| format!("`{}`", n))
            .collect::<Vec<String>>()
            .join(", ");

        let location = ast::RecordLit::cast(diag.fields.to_node(&parse.syntax_node()))
            .and_then(|f| f.type_ref())
            .map(|t| t.syntax().text_range())
            .unwrap_or_else(|| diag.highlight_range());

        MissingFields {
            db,
            diag,
            location,
            missing_fields,
        }
    }
}
