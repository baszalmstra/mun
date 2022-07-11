use crate::{ast, ast::HasDocComments, AstNode, SourceFile};

#[test]
fn doc_comment_struct() {
    let source = SourceFile::parse(
        r#"
        /// This is a doc comment associated with the strukt.
        struct Bla {}
        "#,
    )
    .ok()
    .unwrap();

    let strukt = source
        .syntax
        .descendants()
        .find_map(ast::StructDef::cast)
        .unwrap();
    let comment = strukt.doc_comments().doc_comment_text();
    assert_eq!(
        comment.unwrap(),
        " This is a doc comment associated with the strukt."
    )
}

#[test]
fn doc_comment_function() {
    let source = SourceFile::parse(
        r#"
        /// This is a doc comment associated with the function.
        fn foo() {}
        "#,
    )
    .ok()
    .unwrap();

    let strukt = source
        .syntax
        .descendants()
        .find_map(ast::FunctionDef::cast)
        .unwrap();
    let comment = strukt.doc_comments().doc_comment_text();
    assert_eq!(
        comment.unwrap(),
        " This is a doc comment associated with the function."
    )
}
