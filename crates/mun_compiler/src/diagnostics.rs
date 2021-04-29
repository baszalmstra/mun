use crate::diagnostics_snippets::{emit_hir_diagnostic, emit_syntax_error};
use crate::DisplayColor;
use hir::{DiagnosticSink, HirDatabase};
use std::io::Cursor;

/// Emits all diagnostic messages currently in the database; returns true if errors were
/// emitted.
pub fn emit_diagnostics(
    db: &dyn HirDatabase,
    writer: &mut dyn std::io::Write,
    display_color: DisplayColor,
) -> Result<bool, anyhow::Error> {
    let emit_colors = display_color.should_enable();
    let mut has_error = false;

    for package in hir::Package::all(db) {
        for module in package.modules(db) {
            if let Some(file_id) = module.file_id(db) {
                let parse = db.parse(file_id);
                let source_code = db.file_text(file_id);
                let relative_file_path = db.file_relative_path(file_id);
                let line_index = db.line_index(file_id);

                // Emit all syntax diagnostics
                for syntax_error in parse.errors().iter() {
                    emit_syntax_error(
                        syntax_error,
                        relative_file_path.as_str(),
                        &source_code,
                        &line_index,
                        emit_colors,
                        writer,
                    )?;
                    has_error = true;
                }

                // Emit all HIR diagnostics
                let mut error = None;
                module.diagnostics(
                    db,
                    &mut DiagnosticSink::new(|d| {
                        has_error = true;
                        if let Err(e) = emit_hir_diagnostic(d, db, file_id, emit_colors, writer) {
                            error = Some(e)
                        };
                    }),
                );

                // If an error occurred when emitting HIR diagnostics, return early with the error.
                if let Some(e) = error {
                    return Err(e.into());
                }
            }
        }
    }

    Ok(has_error)
}

/// Returns all diagnostics as a human readable string
pub fn emit_diagnostics_to_string(
    db: &dyn HirDatabase,
    display_color: DisplayColor,
) -> anyhow::Result<Option<String>> {
    let mut compiler_errors: Vec<u8> = Vec::new();
    if !emit_diagnostics(db, &mut Cursor::new(&mut compiler_errors), display_color)? {
        Ok(None)
    } else {
        Ok(Some(String::from_utf8(compiler_errors).map_err(|e| {
            anyhow::anyhow!(
                "could not convert compiler diagnostics to valid UTF8: {}",
                e
            )
        })?))
    }
}

#[cfg(test)]
mod tests {
    use super::emit_diagnostics_to_string;
    use crate::{Config, DisplayColor, Driver, PathOrInline, RelativePathBuf};

    /// Compile passed source code and return all compilation errors
    fn compilation_errors(source_code: &str) -> String {
        let config = Config::default();

        let input = PathOrInline::Inline {
            rel_path: RelativePathBuf::from("main.mun"),
            contents: source_code.to_owned(),
        };

        let (driver, _) = Driver::with_file(config, input).unwrap();
        emit_diagnostics_to_string(driver.database(), DisplayColor::Disable)
            .unwrap()
            .unwrap_or_default()
    }

    #[test]
    fn test_syntax_error() {
        insta::assert_display_snapshot!(compilation_errors("\n\nfn main(\n struct Foo\n"));
    }

    #[test]
    fn test_unresolved_value_error() {
        insta::assert_display_snapshot!(compilation_errors(
            "\n\nfn main() {\nlet b = a;\n\nlet d = c;\n}"
        ));
    }

    #[test]
    fn test_unresolved_type_error() {
        insta::assert_display_snapshot!(compilation_errors(
            "\n\nfn main() {\nlet a = Foo{};\n\nlet b = Bar{};\n}"
        ));
    }

    #[test]
    fn test_expected_function_error() {
        insta::assert_display_snapshot!(compilation_errors(
            "\n\nfn main() {\nlet a = Foo();\n\nlet b = Bar();\n}"
        ));
    }

    #[test]
    fn test_mismatched_type_error() {
        insta::assert_display_snapshot!(compilation_errors(
            "\n\nfn main() {\nlet a: f64 = false;\n\nlet b: bool = 22;\n}"
        ));
    }

    #[test]
    fn test_duplicate_definition_error() {
        insta::assert_display_snapshot!(compilation_errors(
            "\n\nfn foo(){}\n\nfn foo(){}\n\nstruct Bar;\n\nstruct Bar;\n\nfn BAZ(){}\n\nstruct BAZ;"
        ));
    }

    #[test]
    fn test_possibly_uninitialized_variable_error() {
        insta::assert_display_snapshot!(compilation_errors(
            "\n\nfn main() {\nlet a;\nif 5>6 {\na = 5\n}\nlet b = a;\n}"
        ));
    }

    #[test]
    fn test_access_unknown_field_error() {
        insta::assert_display_snapshot!(compilation_errors(
            "\n\nstruct Foo {\ni: bool\n}\n\nfn main() {\nlet a = Foo { i: false };\nlet b = a.t;\n}"
        ));
    }

    #[test]
    fn test_free_type_alias_error() {
        insta::assert_display_snapshot!(compilation_errors("\n\ntype Foo;"));
    }

    #[test]
    fn test_type_alias_target_undeclared_error() {
        insta::assert_display_snapshot!(compilation_errors("\n\ntype Foo = UnknownType;"));
    }

    #[test]
    fn test_cyclic_type_alias_error() {
        insta::assert_display_snapshot!(compilation_errors("\n\ntype Foo = Foo;"));
    }

    #[test]
    fn test_expected_function() {
        insta::assert_display_snapshot!(compilation_errors("\n\nfn foo() { let a = 3; a(); }"));
    }
}
